import os
import re
import subprocess
import altair as alt

from honey_lang import Helper, Input, Output, Function, initialize

initialize(erase_static=False)


def EdbProp(name, **params):
    print(f"[Prop.{name}]")
    for param, typ in params.items():
        print(f'params.{param} = "{typ}"')
    print()


@Helper
def bash(command, redirect_stderr=True):
    command = command.replace("\\\n", "\n")
    command = re.sub(r"\s+", " ", command).strip()

    log("### Running bash command:\n")
    log(command + "\n")
    log("### Output:\n")

    with subprocess.Popen(
        command,
        shell=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT if redirect_stderr else None,
        bufsize=1,
    ) as p:
        if p.stdout:
            for line in p.stdout:
                log(line.removesuffix("\n"))

    log(f"\n### Exit code:\n\n{p.returncode}\n")


@Helper
def already_exists(path):
    if not os.path.isdir(path):
        return False
    for file in os.listdir(path):
        if not file.startswith("."):
            return True
    return False


@Helper
def write_schedule_file(schedule_str, output_dir, *, filename="schedule.halide"):
    os.makedirs(output_dir, exist_ok=True)
    out_path = os.path.join(output_dir, filename)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(schedule_str + "\n")
    return out_path


@Input
class HalideFunc:
    """A Halide function definition

    This is the computation that we want to schedule."""

    name: str
    """Name of the Halide function (e.g. gradient)"""
    definition: str
    """Definition of the function (e.g. gradient(x, y) = x + y)"""
    variables: str
    """Space-separated loop variables of the function (e.g. 'x y')"""


@Output
class Factor:
    """Factor

    The goal of this step is to choose an integer factor. When used by a
    split, this is the extent of the new inner loop; when used by a
    parallelize, this is the task size."""

    path: str

    value: int


@Output
class Directives:
    """Directives

    The goal of this step is to build the chain of scheduling directives
    (parallelize, vectorize, unroll) for a loop that will not be split
    further."""

    path: str

    var: str


@Output
class LoopSchedule:
    """LoopSchedule

    The goal of this step is to schedule one loop: either split it into an
    outer and an inner loop (each of which is then scheduled on its own), or
    stop splitting and attach a chain of directives."""

    path: str

    var: str
    extent: int


@Output
class HalideSchedule:
    """HalideSchedule

    The goal of this step is to produce a schedule for the Halide function
    as a chain of scheduling directives."""

    path: str


# The universe of factors available to choose_factor.
EdbProp("P_Factor", value="Int")

# The loop variables (original and split-created) that directive chains may
# be built for.
EdbProp("P_LoopVar", var="Str")

# Extent-based split legality: a loop `parent` of extent `pextent` may be
# split by `factor` into `outer`/`inner` with extents `oextent`/`iextent`.
# These facts are enumerated by the program generator by walking the
# divide-down closure from each loop's extent, so splitting bottoms out when
# a loop's extent has no legal factor left (rather than at a fixed depth).
EdbProp(
    "P_Split",
    parent="Str",
    pextent="Int",
    factor="Int",
    outer="Str",
    oextent="Int",
    inner="Str",
    iextent="Int",
)

# Ties a function's original loop variables (and their extents) to the
# per-loop schedule slots of build_schedule.
EdbProp("P_FuncLoop", func="Str", pos="Int", var="Str", extent="Int")


@Function(
    "P_Factor { value = ret.value }",
)
def choose_factor(__hb_ret: Factor):
    """Choose a factor

    Choose an integer factor. When used by a split, this is the extent of
    the new inner loop; when used by a parallelize, this is the task size."""
    print(f"Factor: {__hb_ret.value}.")


@Function(
    "P_LoopVar { var = ret.var }",
)
def leaf(__hb_ret: Directives):
    """Plain loop (end of directives)

    End the directive chain for this loop."""
    __hb_ret.directives = []
    print(f"Loop '{__hb_ret.var}': end of directives.")


@Function(
    "ret.var = rest.var",
)
def parallelize(__hb_f: Factor, __hb_rest: Directives, __hb_ret: Directives):
    """Parallelize this loop

    Distribute the iterations of this loop across threads, processing them
    in tasks of the given factor (Halide's parallel(var, task_size))."""
    __hb_ret.directives = [
        f"parallel({__hb_ret.var}, {__hb_f.value})"
    ] + __hb_rest.directives
    print(f"Parallelize '{__hb_ret.var}' with task size {__hb_f.value}.")


@Function(
    "ret.var = rest.var",
)
def vectorize(__hb_rest: Directives, __hb_ret: Directives):
    """Vectorize this loop

    Execute this loop's iterations as a single vector operation. The vector
    width is this loop's extent, so this is usually applied to an inner loop
    produced by a split (Halide's vectorize(var))."""
    __hb_ret.directives = [f"vectorize({__hb_ret.var})"] + __hb_rest.directives
    print(f"Vectorize '{__hb_ret.var}'.")


@Function(
    "ret.var = rest.var",
)
def unroll(__hb_rest: Directives, __hb_ret: Directives):
    """Unroll this loop

    Unroll this loop completely over its extent (Halide's unroll(var))."""
    __hb_ret.directives = [f"unroll({__hb_ret.var})"] + __hb_rest.directives
    print(f"Unroll '{__hb_ret.var}'.")


@Function(
    "ret.var = q.var",
)
def inject(__hb_q: Directives, __hb_ret: LoopSchedule):
    """Do not split this loop

    Stop splitting this loop and schedule it with the given directive
    chain."""
    __hb_ret.splits = []
    __hb_ret.directives = __hb_q.directives
    print(f"Loop '{__hb_ret.var}' (extent {__hb_ret.extent}): not split further.")


@Function(
    "P_Split { parent = ret.var, pextent = ret.extent, factor = f.value, outer = outer.var, oextent = outer.extent, inner = inner.var, iextent = inner.extent }",
)
def simple_split(
    __hb_f: Factor,
    __hb_outer: LoopSchedule,
    __hb_inner: LoopSchedule,
    __hb_ret: LoopSchedule,
):
    """Split this loop

    Split this loop into an outer and an inner loop, where the inner loop
    does factor-many iterations (Halide's split(var, outer, inner, factor)).
    Each new loop is then scheduled on its own."""
    __hb_ret.splits = (
        [f"split({__hb_ret.var}, {__hb_outer.var}, {__hb_inner.var}, {__hb_f.value})"]
        + __hb_outer.splits
        + __hb_inner.splits
    )
    __hb_ret.directives = __hb_outer.directives + __hb_inner.directives
    print(
        f"Split '{__hb_ret.var}' (extent {__hb_ret.extent}) into "
        f"'{__hb_outer.var}' (outer, extent {__hb_outer.extent}) and "
        f"'{__hb_inner.var}' (inner, extent {__hb_inner.extent}) "
        f"with factor {__hb_f.value}."
    )


@Function(
    "P_FuncLoop { func = func.name, pos = 0, var = a0.var, extent = a0.extent }",
    "P_FuncLoop { func = func.name, pos = 1, var = a1.var, extent = a1.extent }",
)
def build_schedule(
    __hb_func: HalideFunc,
    __hb_a0: LoopSchedule,
    __hb_a1: LoopSchedule,
    __hb_ret: HalideSchedule,
):
    """build_schedule

    Combine the per-loop schedules of a two-dimensional Halide function into
    the final schedule; all splits first"""
    schedule = __hb_func.name
    for op in __hb_a0.splits + __hb_a1.splits + __hb_a0.directives + __hb_a1.directives:
        schedule += f".{op}"
    write_schedule_file(schedule, __hb_ret.path)
    print(schedule)
