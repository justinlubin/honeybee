import os
import subprocess
import glob
import datetime
import polars as pl
import json
import re
import altair as alt
from itertools import permutations
from scorch.compiler.cin import IndexVar, TensorVar, ForAll
from scorch.compiler.scheduler import Scheduler

from honey_lang import Helper, Input, Output, Function, initialize

initialize()

@Helper
class Dir:
    stage = 1

    def make(name):
        time = datetime.datetime.today().strftime("%Y-%m-%d-%H-%M-%S")
        dir = f"output-{time}/{Dir.stage * 10:03d}-{name}"
        os.makedirs(dir, exist_ok=True)
        Dir.stage += 1
        return dir


@Helper
def carry_over(src_object, dst_object, *, file=None):
    def carry_one(file):
        src = f"{src_object.path}/{file}"
        dst = f"{dst_object.path}/{file}"
        if os.path.islink(src):
            src = os.readlink(src)
        os.symlink(src=src, dst=dst)

    if file is None:
        for file in os.listdir(src_object.path):
            carry_one(file)
    else:
        carry_one(file)


@Helper
def write_schedule_json(schedule_dict, output_dir, *, filename="schedule.json"):
    os.makedirs(output_dir, exist_ok=True)
    out_path = os.path.join(output_dir, filename)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(schedule_dict, f, indent=2, sort_keys=True)
    return out_path


@Helper
class ScheduleState:
    stream_level = None
    par_factor = None
    stream_shape = None
    block_sparse = None
    dataflow_ordering = None

    @classmethod
    def set_values(
        cls,
        *,
        stream_level=None,
        par_factor=None,
        stream_shape=None,
        block_sparse=None,
        dataflow_ordering=None,
    ):
        if stream_level is not None:
            cls.stream_level = stream_level
        if par_factor is not None:
            cls.par_factor = par_factor
        if stream_shape is not None:
            cls.stream_shape = stream_shape
        if block_sparse is not None:
            cls.block_sparse = block_sparse
        if dataflow_ordering is not None:
            cls.dataflow_ordering = dataflow_ordering

    @classmethod
    def as_dict(cls):
        return {
            "stream-parallelizer": {
                "stream-level": cls.stream_level,
                "par-factor": cls.par_factor,
            },
            "stream-vectorizer": {
                "stream-shape": cls.stream_shape,
                "enable-block-sparse": cls.block_sparse,
            },
            "dataflow-ordering": cls.dataflow_ordering,
        }

    @classmethod
    def missing_fields(cls):
        missing = []
        if cls.stream_level is None:
            missing.append("stream_level")
        if cls.par_factor is None:
            missing.append("par_factor")
        if cls.stream_shape is None:
            missing.append("stream_shape")
        if cls.block_sparse is None:
            missing.append("block_sparse")
        if cls.dataflow_ordering is None:
            missing.append("dataflow_ordering")
        return missing


@Helper
def parse_tensor_formats(spec):
    formats = {}
    for entry in spec.split():
        name, sep, fmt = entry.partition(":")
        if not name or not sep or not fmt:
            raise ValueError(
                f"Malformed tensor_formats entry '{entry}'; expected 'name:format'"
            )
        formats[name] = fmt
    return formats


@Helper
def scorch_loop_order(cin_expression, formats):
    lhs_str, rhs_str = (s.strip() for s in cin_expression.split("="))
    term = re.compile(r"(\w+)\(([\w,\s]+)\)")

    def parse(s, t):
        return [
            (m.group(1), [i.strip() for i in m.group(2).split(",")])
            for m in t.finditer(s)
        ]

    ((out_name, out_idx),) = parse(lhs_str, term)
    rhs_terms = parse(rhs_str, term)

    missing = [
        name for name, _ in [(out_name, out_idx), *rhs_terms] if name not in formats
    ]
    if missing:
        raise ValueError(
            "No format given for tensor(s) " + ", ".join(sorted(set(missing)))
        )

    index_vars = {}
    for _, idxs in [(out_name, out_idx), *rhs_terms]:
        for i in idxs:
            index_vars.setdefault(i, IndexVar(i))

    def access(name, idxs):
        tv = TensorVar(name, fmt=formats[name])
        keys = [index_vars[i] for i in idxs]
        return tv[keys[0]] if len(keys) == 1 else tv[(tuple(keys))]

    rhs = None
    for name, idxs in rhs_terms:
        a = access(name, idxs)
        rhs = a if rhs is None else rhs * a

    out_tv = TensorVar(out_name, fmt=formats[out_name])
    out_lhs = [index_vars[i] for i in out_idx]
    out_tv[out_lhs[0] if len(out_lhs) == 1 else tuple(out_lhs)] = rhs

    cin = out_tv._assignment
    for iv in reversed(list(index_vars.values())):
        cin = ForAll(iv, cin)

    return ",".join([iv.name for iv in Scheduler.select_loop_order(cin)])


################################################################################
# %% FuseFlow Schedule
@Input
class MlirProgram:
    """An MLIR program in your filesystem

    This is the program that we use to generate the schedule."""

    path: str
    """Path to the MLIR program"""
    num_loops: int
    """Number of loops identified by FuseFlow compiler"""

    cin_expression: str
    """The tensor index expression for the given operation

    @example:tOut1(i0, i3) = t0(i0, i1) * t1(i1, i2) * t2(i2, i3)"""

    tensor_formats: str
    """Per-tensor sparse formats, as space-separated 'name:format' pairs"""


@Input
class LoopOrderOption:
    """LoopOrderOption

    A loop ordering option generated by the FuseFlow compiler."""

    path: str
    """Path to the MLIR program this ordering applies to"""
    order: str
    """Loop order string"""


@Output
class VectorizationPass:
    """VectorizationPass

    The goal of this step is to produce the schedule for the parallelization
    pass in the FuseFlow compiler"""

    path: str
    mlir_path: str
    num_loops: int


@Output
class StreamShapeChoice:
    """StreamShapeChoice

    The goal of this step is to choose a stream shape for vectorization."""

    path: str

    stream_shape: int
    mlir_path: str
    num_loops: int


@Output
class BlockSparseChoice:
    """BlockSparseChoice

    The goal of this step is to choose whether block-sparse vectorization is enabled."""

    path: str

    block_sparse: bool
    mlir_path: str
    num_loops: int


@Output
class StreamLevelChoice:
    """StreamLevelChoice

    The goal of this step is to choose a stream level for parallelization."""

    path: str

    stream_level: int
    mlir_path: str


@Output
class ParallelizationPass:
    """ParallelizationPass

    The goal of this step is to produce the schedule for the parallelization
    pass in the FuseFlow compiler"""

    path: str
    mlir_path: str


@Output
class ParFactorChoice:
    """ParFactorChoice

    The goal of this step is to choose a parallelization factor."""

    path: str

    par_factor: int
    mlir_path: str


@Output
class LoopOrderChoice:
    """LoopOrderChoice

    The goal of this step is to choose a dataflow loop ordering."""

    path: str

    order: str


@Output
class FuseFlowSchedule:
    """FuseFlowSchedule

    The goal of this step is to produce a schedule for the FuseFlow compiler
    that sets the parallelization and vectorization pass parameters"""

    path: str


@Function()
def default_schedule(__hb_ret: FuseFlowSchedule):
    """schedule

    The function that produces a schedule."""
    ScheduleState.set_values(
        stream_level=0,
        par_factor=1,
        stream_shape=16,
        block_sparse=False,
        dataflow_ordering="",
    )
    schedule = ScheduleState.as_dict()
    write_schedule_json(schedule, __hb_ret.path)
    print(json.dumps(schedule, indent=2, sort_keys=True))


@Function()
def build_schedule(__hb_order: LoopOrderChoice, __hb_ret: FuseFlowSchedule):
    missing = ScheduleState.missing_fields()
    if missing:
        raise RuntimeError("schedule state incomplete; missing: " + ", ".join(missing))
    schedule = ScheduleState.as_dict()
    write_schedule_json(schedule, __hb_ret.path)
    print(json.dumps(schedule, indent=2, sort_keys=True))


@Function(
    "ret.stream_level = 0",
    "ret.stream_level < vec.num_loops",
    "ret.mlir_path = vec.mlir_path",
)
def choose_default_stream_level(
    __hb_vec: VectorizationPass, __hb_ret: StreamLevelChoice
):
    ScheduleState.set_values(stream_level=0)
    print("Choose default stream level (0).")


@Function(
    "ret.stream_level = 1",
    "ret.stream_level < vec.num_loops",
    "ret.mlir_path = vec.mlir_path",
)
def choose_stream_level_1(__hb_vec: VectorizationPass, __hb_ret: StreamLevelChoice):
    ScheduleState.set_values(stream_level=1)
    print("Choose stream level 1.")


@Function(
    "ret.stream_level = 2",
    "ret.stream_level < vec.num_loops",
    "ret.mlir_path = vec.mlir_path",
)
def choose_stream_level_2(__hb_vec: VectorizationPass, __hb_ret: StreamLevelChoice):
    ScheduleState.set_values(stream_level=2)
    print("Choose stream level 2.")


@Function(
    "ret.stream_level = 4",
    "ret.stream_level < vec.num_loops",
    "ret.mlir_path = vec.mlir_path",
)
def choose_stream_level_4(__hb_vec: VectorizationPass, __hb_ret: StreamLevelChoice):
    ScheduleState.set_values(stream_level=4)
    print("Choose stream level 4.")


@Function(
    "ret.stream_level = 8",
    "ret.stream_level < vec.num_loops",
    "ret.mlir_path = vec.mlir_path",
)
def choose_stream_level_8(__hb_vec: VectorizationPass, __hb_ret: StreamLevelChoice):
    ScheduleState.set_values(stream_level=8)
    print("Choose stream level 8.")


@Function(
    "ret.stream_level = 16",
    "ret.stream_level < vec.num_loops",
    "ret.mlir_path = vec.mlir_path",
)
def choose_stream_level_16(__hb_vec: VectorizationPass, __hb_ret: StreamLevelChoice):
    ScheduleState.set_values(stream_level=16)
    print("Choose stream level 16.")


@Function(
    "ret.par_factor = 1",
    "ret.mlir_path = level.mlir_path",
)
def choose_default_par_factor(__hb_level: StreamLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_values(par_factor=1)
    print("Choose default par factor (1).")


@Function(
    "ret.par_factor = 2",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_2(__hb_level: StreamLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_values(par_factor=2)
    print("Choose par factor 2.")


@Function(
    "ret.par_factor = 4",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_4(__hb_level: StreamLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_values(par_factor=4)
    print("Choose par factor 4.")


@Function(
    "ret.par_factor = 8",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_8(__hb_level: StreamLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_values(par_factor=8)
    print("Choose par factor 8.")


@Function(
    "ret.par_factor = 16",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_16(__hb_level: StreamLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_values(par_factor=16)
    print("Choose par factor 16.")


@Function(
    "ret.stream_shape = 16",
    "ret.mlir_path = mlir.path",
    "ret.num_loops = mlir.num_loops",
)
def choose_default_stream_shape(__hb_mlir: MlirProgram, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=16)
    print("Choose default stream shape (16).")


@Function(
    "ret.stream_shape = 1",
    "ret.mlir_path = mlir.path",
    "ret.num_loops = mlir.num_loops",
)
def choose_stream_shape_1(__hb_mlir: MlirProgram, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=1)
    print("Choose stream shape 1.")


@Function(
    "ret.stream_shape = 2",
    "ret.mlir_path = mlir.path",
    "ret.num_loops = mlir.num_loops",
)
def choose_stream_shape_2(__hb_mlir: MlirProgram, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=2)
    print("Choose stream shape 2.")


@Function(
    "ret.stream_shape = 4",
    "ret.mlir_path = mlir.path",
    "ret.num_loops = mlir.num_loops",
)
def choose_stream_shape_4(__hb_mlir: MlirProgram, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=4)
    print("Choose stream shape 4.")


@Function(
    "ret.stream_shape = 8",
    "ret.mlir_path = mlir.path",
    "ret.num_loops = mlir.num_loops",
)
def choose_stream_shape_8(__hb_mlir: MlirProgram, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=8)
    print("Choose stream shape 8.")


@Function(
    "ret.stream_shape = 16",
    "ret.mlir_path = mlir.path",
    "ret.num_loops = mlir.num_loops",
)
def choose_stream_shape_16(__hb_mlir: MlirProgram, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=16)
    print("Choose stream shape 16.")


@Function(
    "ret.block_sparse = false",
    "ret.mlir_path = shape.mlir_path",
    "ret.num_loops = shape.num_loops",
)
def choose_default_block_sparse(
    __hb_shape: StreamShapeChoice, __hb_ret: BlockSparseChoice
):
    ScheduleState.set_values(block_sparse=False)
    print("Choose default block sparse (false).")


@Function(
    "ret.block_sparse = true",
    "ret.mlir_path = shape.mlir_path",
    "ret.num_loops = shape.num_loops",
)
def choose_block_sparse_true(
    __hb_shape: StreamShapeChoice, __hb_ret: BlockSparseChoice
):
    ScheduleState.set_values(block_sparse=True)
    print("Choose block sparse (true).")


@Function(
    "ret.mlir_path = block.mlir_path",
    "ret.num_loops = block.num_loops",
)
def vectorization(__hb_block: BlockSparseChoice, __hb_ret: VectorizationPass):
    print("Vectorization.")


@Function(
    "ret.mlir_path = par.mlir_path",
)
def parallelization(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")


@Function(
    "P_LoopOrderOption { path = pass.mlir_path, order = ret.order }",
)
def choose_loop_order(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print("Choose dataflow loop order.")


@Function(
    'ret.order = ""',
)
def use_scorch_loop_order(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Let Scorch choose the dataflow order"""
    expr = MLIR_PROGRAM.cin_expression
    formats = parse_tensor_formats(MLIR_PROGRAM.tensor_formats)
    order = scorch_loop_order(expr, formats)
    ScheduleState.set_values(dataflow_ordering=order)
    print(f"Choose scorch-generated dataflow loop order: {order}")


MAX_LOOPS = 4

LOOP_ORDER_CONDITIONS = [
    (
        f"P_MlirProgram {{ path = pass.mlir_path, num_loops = {n}, "
        f"cin_expression = _, tensor_formats = _ }}",
        f'ret.order = "{" ".join(p)}"',
    )
    for n in range(1, MAX_LOOPS + 1)
    for p in permutations([f"i{k}" for k in range(n)])
]


@Function(*LOOP_ORDER_CONDITIONS[0])
def choose_loop_order_0(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[1])
def choose_loop_order_1(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[2])
def choose_loop_order_2(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[3])
def choose_loop_order_3(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[4])
def choose_loop_order_4(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[5])
def choose_loop_order_5(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[6])
def choose_loop_order_6(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[7])
def choose_loop_order_7(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[8])
def choose_loop_order_8(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[9])
def choose_loop_order_9(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[10])
def choose_loop_order_10(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[11])
def choose_loop_order_11(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[12])
def choose_loop_order_12(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[13])
def choose_loop_order_13(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[14])
def choose_loop_order_14(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[15])
def choose_loop_order_15(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[16])
def choose_loop_order_16(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[17])
def choose_loop_order_17(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[18])
def choose_loop_order_18(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[19])
def choose_loop_order_19(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[20])
def choose_loop_order_20(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[21])
def choose_loop_order_21(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[22])
def choose_loop_order_22(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[23])
def choose_loop_order_23(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[24])
def choose_loop_order_24(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[25])
def choose_loop_order_25(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[26])
def choose_loop_order_26(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[27])
def choose_loop_order_27(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[28])
def choose_loop_order_28(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[29])
def choose_loop_order_29(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[30])
def choose_loop_order_30(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[31])
def choose_loop_order_31(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")


@Function(*LOOP_ORDER_CONDITIONS[32])
def choose_loop_order_32(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    """Choose dataflow loop order"""
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print(f"Choose dataflow loop order ({__hb_ret.order}).")
