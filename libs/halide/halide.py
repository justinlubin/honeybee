import os
import glob
import re
import subprocess
import altair as alt
from itertools import permutations

from honey_lang import Helper, Input, Output, Function, initialize

initialize(erase_static=False)


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
def stem(path):
    ret = path
    while True:
        ret, ext = os.path.splitext(os.path.basename(ret))
        if not ext:
            return ret


@Helper
def carry_over(source_glob, destination_folder):
    os.makedirs(destination_folder, exist_ok=True)
    if "*" not in source_glob:
        source_glob += "/*"
    sources = glob.glob(source_glob)
    assert sources != [], f"Pattern '{source_glob}' did not match any files"
    for src in sorted(sources):
        basename = os.path.basename(src)
        src = os.path.relpath(src, start=destination_folder)
        dst = f"{destination_folder}/{basename}"
        os.symlink(src=src, dst=dst)


@Helper
def link(src, dst):
    assert os.path.exists(src), f"Cannot find file/folder '{src}'"
    destination_folder = os.path.dirname(dst)
    os.makedirs(destination_folder, exist_ok=True)
    src = os.path.relpath(src, start=destination_folder)
    os.symlink(src=src, dst=dst)


@Helper
def shared():
    path = "output/shared"
    os.makedirs(path, exist_ok=True)
    return path


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


@Helper
class ScheduleState:
    func_name = None
    variables = None
    reorder = None
    tile = None
    vectorize_factor = None
    par_factor = None
    unroll_factor = None

    @classmethod
    def set_values(
        cls,
        *,
        func_name=None,
        variables=None,
        reorder=None,
        tile=None,
        vectorize_factor=None,
        par_factor=None,
        unroll_factor=None,
    ):
        if func_name is not None:
            cls.func_name = func_name
        if variables is not None:
            cls.variables = variables
        if reorder is not None:
            cls.reorder = reorder
        if tile is not None:
            cls.tile = tile
        if vectorize_factor is not None:
            cls.vectorize_factor = vectorize_factor
        if par_factor is not None:
            cls.par_factor = par_factor
        if unroll_factor is not None:
            cls.unroll_factor = unroll_factor

    @classmethod
    def missing_fields(cls):
        missing = []
        if cls.func_name is None:
            missing.append("func_name")
        if cls.reorder is None:
            missing.append("reorder")
        if cls.tile is None:
            missing.append("tile")
        if cls.vectorize_factor is None:
            missing.append("vectorize_factor")
        if cls.par_factor is None:
            missing.append("par_factor")
        if cls.unroll_factor is None:
            missing.append("unroll_factor")
        return missing

    @classmethod
    def as_schedule(cls):
        schedule = cls.func_name
        if cls.reorder:
            schedule += f".reorder({', '.join(cls.reorder.split())})"
        if cls.tile:
            schedule += ".tile()"
        if cls.vectorize_factor:
            schedule += f".vectorize({cls.vectorize_factor})"
        if cls.par_factor:
            schedule += f".parallel({cls.par_factor})"
        if cls.unroll_factor:
            schedule += f".unroll({cls.unroll_factor})"
        return schedule


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
class TileChoice:
    """TileChoice

    The goal of this step is to choose whether to tile the loop nest."""

    path: str

    tile: bool
    func_name: str
    variables: str


@Output
class VectorizeChoice:
    """VectorizeChoice

    The goal of this step is to choose a vectorization factor for the innermost
    loop (0 means no vectorization)."""

    path: str

    vectorize_factor: int
    func_name: str
    variables: str


@Output
class ParallelChoice:
    """ParallelChoice

    The goal of this step is to choose a parallelization factor for the
    outermost loop (0 means no parallelization)."""

    path: str

    par_factor: int
    func_name: str
    variables: str


@Output
class UnrollChoice:
    """UnrollChoice

    The goal of this step is to choose an unroll factor for the innermost loop
    (0 means no unrolling)."""

    path: str

    unroll_factor: int
    func_name: str
    variables: str


@Output
class ReorderChoice:
    """ReorderChoice

    The goal of this step is to choose a reordering (permutation) of the loop
    variables, or keep the default order."""

    path: str

    perm_index: int
    func_name: str
    variables: str


@Output
class HalideSchedule:
    """HalideSchedule

    The goal of this step is to produce a schedule for the Halide function
    as a chain of scheduling directives."""

    path: str


@Function(
    "ret.tile = true",
    "ret.func_name = func.name",
    "ret.variables = func.variables",
)
def choose_tile(__hb_func: HalideFunc, __hb_ret: TileChoice):
    ScheduleState.set_values(func_name=__hb_func.name, tile=True)
    print(f"Tile '{__hb_func.name}': yes.")


@Function(
    "ret.tile = false",
    "ret.func_name = func.name",
    "ret.variables = func.variables",
)
def choose_no_tile(__hb_func: HalideFunc, __hb_ret: TileChoice):
    ScheduleState.set_values(func_name=__hb_func.name, tile=False)
    print(f"Tile '{__hb_func.name}': no.")


@Function(
    "ret.vectorize_factor = 0",
    "ret.func_name = tile.func_name",
    "ret.variables = tile.variables",
)
def choose_no_vectorize(__hb_tile: TileChoice, __hb_ret: VectorizeChoice):
    ScheduleState.set_values(vectorize_factor=0)
    print(f"Vectorize '{__hb_tile.func_name}': no.")


@Function(
    "ret.vectorize_factor = 2",
    "ret.func_name = tile.func_name",
    "ret.variables = tile.variables",
)
def choose_vectorize_2(__hb_tile: TileChoice, __hb_ret: VectorizeChoice):
    ScheduleState.set_values(vectorize_factor=2)
    print(f"Vectorize '{__hb_tile.func_name}' by 2.")


@Function(
    "ret.vectorize_factor = 4",
    "ret.func_name = tile.func_name",
    "ret.variables = tile.variables",
)
def choose_vectorize_4(__hb_tile: TileChoice, __hb_ret: VectorizeChoice):
    ScheduleState.set_values(vectorize_factor=4)
    print(f"Vectorize '{__hb_tile.func_name}' by 4.")


@Function(
    "ret.vectorize_factor = 8",
    "ret.func_name = tile.func_name",
    "ret.variables = tile.variables",
)
def choose_vectorize_8(__hb_tile: TileChoice, __hb_ret: VectorizeChoice):
    ScheduleState.set_values(vectorize_factor=8)
    print(f"Vectorize '{__hb_tile.func_name}' by 8.")


@Function(
    "ret.par_factor = 0",
    "ret.func_name = vec.func_name",
    "ret.variables = vec.variables",
)
def choose_no_parallel(__hb_vec: VectorizeChoice, __hb_ret: ParallelChoice):
    ScheduleState.set_values(par_factor=0)
    print(f"Parallelize '{__hb_vec.func_name}': no.")


@Function(
    "ret.par_factor = 2",
    "ret.func_name = vec.func_name",
    "ret.variables = vec.variables",
)
def choose_parallel_2(__hb_vec: VectorizeChoice, __hb_ret: ParallelChoice):
    ScheduleState.set_values(par_factor=2)
    print(f"Parallelize '{__hb_vec.func_name}' by 2.")


@Function(
    "ret.par_factor = 4",
    "ret.func_name = vec.func_name",
    "ret.variables = vec.variables",
)
def choose_parallel_4(__hb_vec: VectorizeChoice, __hb_ret: ParallelChoice):
    ScheduleState.set_values(par_factor=4)
    print(f"Parallelize '{__hb_vec.func_name}' by 4.")


@Function(
    "ret.par_factor = 8",
    "ret.func_name = vec.func_name",
    "ret.variables = vec.variables",
)
def choose_parallel_8(__hb_vec: VectorizeChoice, __hb_ret: ParallelChoice):
    ScheduleState.set_values(par_factor=8)
    print(f"Parallelize '{__hb_vec.func_name}' by 8.")


@Function(
    "ret.unroll_factor = 0",
    "ret.func_name = par.func_name",
    "ret.variables = par.variables",
)
def choose_no_unroll(__hb_par: ParallelChoice, __hb_ret: UnrollChoice):
    ScheduleState.set_values(unroll_factor=0)
    print(f"Unroll '{__hb_par.func_name}': no.")


@Function(
    "ret.unroll_factor = 2",
    "ret.func_name = par.func_name",
    "ret.variables = par.variables",
)
def choose_unroll_2(__hb_par: ParallelChoice, __hb_ret: UnrollChoice):
    ScheduleState.set_values(unroll_factor=2)
    print(f"Unroll '{__hb_par.func_name}' by 2.")


@Function(
    "ret.unroll_factor = 4",
    "ret.func_name = par.func_name",
    "ret.variables = par.variables",
)
def choose_unroll_4(__hb_par: ParallelChoice, __hb_ret: UnrollChoice):
    ScheduleState.set_values(unroll_factor=4)
    print(f"Unroll '{__hb_par.func_name}' by 4.")


@Function(
    "ret.unroll_factor = 8",
    "ret.func_name = par.func_name",
    "ret.variables = par.variables",
)
def choose_unroll_8(__hb_par: ParallelChoice, __hb_ret: UnrollChoice):
    ScheduleState.set_values(unroll_factor=8)
    print(f"Unroll '{__hb_par.func_name}' by 8.")


@Function(
    "ret.perm_index = 0",
    "ret.func_name = prev.func_name",
    "ret.variables = prev.variables",
)
def choose_no_reorder(__hb_prev: UnrollChoice, __hb_ret: ReorderChoice):
    """Keep the default loop order (no reorder)."""
    ScheduleState.set_values(variables=__hb_prev.variables, reorder="")
    print(f"Reorder '{__hb_prev.func_name}': no.")


@Function(
    "ret.perm_index = 1",
    "ret.func_name = prev.func_name",
    "ret.variables = prev.variables",
)
def choose_reorder_1(__hb_prev: UnrollChoice, __hb_ret: ReorderChoice):
    """Reorder the loop variables to permutation 1."""
    perms = list(permutations(__hb_prev.variables.split()))
    order = " ".join(perms[__hb_ret.perm_index])
    ScheduleState.set_values(variables=__hb_prev.variables, reorder=order)
    print(f"Reorder '{__hb_prev.func_name}' to ({order}).")


@Function()
def build_schedule(__hb_reorder: ReorderChoice, __hb_ret: HalideSchedule):
    """build_schedule

    The function that produces the final Halide schedule."""
    missing = ScheduleState.missing_fields()
    if missing:
        raise RuntimeError("schedule state incomplete; missing: " + ", ".join(missing))
    schedule = ScheduleState.as_schedule()
    write_schedule_file(schedule, __hb_ret.path)
    print(schedule)
