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
    further. Which loop this chain applies to is determined by where it sits
    in the schedule expression; loop variable names are assigned
    automatically when the final schedule is built."""

    path: str


@Output
class LoopSchedule:
    """LoopSchedule

    The goal of this step is to schedule one loop: either split it into an
    outer and an inner loop (each of which is then scheduled on its own), or
    stop splitting and attach a chain of directives. Which loop this
    schedules is determined by where it sits in the schedule expression: the
    slots of build_schedule are the function's original loops, and a split's
    outer/inner arguments are the two loops it creates (named v_o and v_i
    for a split of v when the final schedule is built)."""

    path: str

    extent: int


@Output
class HalideSchedule:
    """HalideSchedule

    The goal of this step is to produce a schedule for the Halide function
    as a chain of scheduling directives."""

    path: str


# The universe of factors available to choose_factor.
EdbProp("P_Factor", value="Int")

EdbProp("P_Div", dividend="Int", divisor="Int", quotient="Int")

# The extents of a function's original loop variables, by position.
EdbProp("P_FuncLoop", func="Str", pos="Int", extent="Int")


@Function(
    "P_Factor { value = ret.value }",
)
def choose_factor(__hb_ret: Factor):
    """Choose a factor

    Choose an integer factor. When used by a split, this is the extent of
    the new inner loop; when used by a parallelize, this is the task size."""
    print(f"Factor: {__hb_ret.value}.")


@Function()
def leaf(__hb_ret: Directives):
    """Plain loop (end of directives)

    End the directive chain for this loop."""
    __hb_ret.chain = []
    print("End of directives.")


@Function()
def parallelize(__hb_f: Factor, __hb_rest: Directives, __hb_ret: Directives):
    """Parallelize this loop

    Distribute the iterations of this loop across threads, processing them
    in tasks of the given factor (Halide's parallel(var, task_size))."""
    __hb_ret.chain = [("parallel", __hb_f.value)] + __hb_rest.chain
    print(f"Parallelize this loop with task size {__hb_f.value}.")


@Function()
def vectorize(__hb_rest: Directives, __hb_ret: Directives):
    """Vectorize this loop

    Execute this loop's iterations as a single vector operation. The vector
    width is this loop's extent, so this is usually applied to an inner loop
    produced by a split (Halide's vectorize(var))."""
    __hb_ret.chain = [("vectorize", None)] + __hb_rest.chain
    print("Vectorize this loop.")


@Function()
def unroll(__hb_rest: Directives, __hb_ret: Directives):
    """Unroll this loop

    Unroll this loop completely over its extent (Halide's unroll(var))."""
    __hb_ret.chain = [("unroll", None)] + __hb_rest.chain
    print("Unroll this loop.")


@Function()
def inject(__hb_q: Directives, __hb_ret: LoopSchedule):
    """Do not split this loop

    Stop splitting this loop and schedule it with the given directive
    chain."""
    __hb_ret.tree = ("leaf", __hb_q.chain)
    print(f"Loop of extent {__hb_ret.extent}: not split further.")


@Function(
    "P_Div { dividend = ret.extent, divisor = f.value, quotient = outer.extent }",
    "inner.extent = f.value",
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
    Each new loop is then scheduled on its own; when the final schedule is
    built, a split of loop v names its new loops v_o and v_i."""
    __hb_ret.tree = ("split", __hb_f.value, __hb_outer.tree, __hb_inner.tree)
    print(
        f"Split this loop (extent {__hb_ret.extent}) into an outer loop "
        f"(extent {__hb_outer.extent}) and an inner loop "
        f"(extent {__hb_inner.extent}) with factor {__hb_f.value}."
    )


@Function(
    "P_FuncLoop { func = func.name, pos = 0, extent = a0.extent }",
    "P_FuncLoop { func = func.name, pos = 1, extent = a1.extent }",
)
def build_schedule(
    __hb_func: HalideFunc,
    __hb_a0: LoopSchedule,
    __hb_a1: LoopSchedule,
    __hb_ret: HalideSchedule,
):
    """build_schedule

    Combine the per-loop schedules of a two-dimensional Halide function into
    the final schedule: all splits first (parents before children), then all
    directives. Loop variable names are assigned here, top-down: a split of
    loop v names its new loops v_o and v_i."""
    splits = []
    directives = []

    def walk(var, tree):
        if tree[0] == "split":
            _, factor, outer, inner = tree
            splits.append(f"split({var}, {var}_o, {var}_i, {factor})")
            walk(f"{var}_o", outer)
            walk(f"{var}_i", inner)
        else:
            _, chain = tree
            for op, arg in chain:
                if arg is None:
                    directives.append(f"{op}({var})")
                else:
                    directives.append(f"{op}({var}, {arg})")

    loop_vars = __hb_func.variables.split()
    for var, tree in zip(loop_vars, [__hb_a0.tree, __hb_a1.tree]):
        walk(var, tree)

    schedule = __hb_func.name
    for op in splits + directives:
        schedule += f".{op}"
    write_schedule_file(schedule, __hb_ret.path)
    print(schedule)
