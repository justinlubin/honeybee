import os
import datetime
import polars as pl
import json

from honey_lang import Helper, Input, Output, Function, __hb_bash

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
    par_factors = None
    stream_shape = None
    block_sparse = None
    dataflow_ordering = None
    sparse_annotations = None

    @classmethod
    def initialize_parallel_schedule(cls, *, num_loops, max_levels=16):
        if num_loops <= 0:
            raise ValueError("num_loops must be positive")
        if num_loops > max_levels:
            raise ValueError(
                f"num_loops={num_loops} exceeds maximum supported stream levels ({max_levels})"
            )
        cls.par_factors = [1 for _ in range(num_loops)]

    @classmethod
    def set_par_factor_for_level(cls, *, stream_level, par_factor):
        if cls.par_factors is None:
            raise RuntimeError("parallel schedule not initialized")
        if stream_level < 0 or stream_level >= len(cls.par_factors):
            raise ValueError("stream_level out of bounds")
        cls.par_factors[stream_level] = par_factor

    @classmethod
    def _ensure_tensor_entry(cls, tensor_name):
        if cls.sparse_annotations is None:
            cls.sparse_annotations = {}
        if tensor_name not in cls.sparse_annotations:
            cls.sparse_annotations[tensor_name] = {"format": None, "indices": {}}
        return cls.sparse_annotations[tensor_name]

    @classmethod
    def annotate_tensor_index(cls, tensor_name, index_name, is_sparse):
        entry = cls._ensure_tensor_entry(tensor_name)
        entry["indices"][index_name] = is_sparse
        if entry["format"] is None:
            entry["format"] = "custom"

    @classmethod
    def annotate_tensor_format(cls, tensor_name, format_name, index_names, sparsity_pattern):
        if len(index_names) != len(sparsity_pattern):
            raise ValueError("index_names and sparsity_pattern must have matching length")
        entry = cls._ensure_tensor_entry(tensor_name)
        entry["format"] = format_name
        entry["indices"] = {
            name: bool(flag) for name, flag in zip(index_names, sparsity_pattern)
        }

    @classmethod
    def set_values(
        cls,
        *,
        par_factors=None,
        stream_shape=None,
        block_sparse=None,
        dataflow_ordering=None,
        sparse_annotations=None,
    ):
        if par_factors is not None:
            cls.par_factors = par_factors
        if stream_shape is not None:
            cls.stream_shape = stream_shape
        if block_sparse is not None:
            cls.block_sparse = block_sparse
        if dataflow_ordering is not None:
            cls.dataflow_ordering = dataflow_ordering
        if sparse_annotations is not None:
            cls.sparse_annotations = sparse_annotations

    @classmethod
    def as_dict(cls):
        return {
            "stream-parallelizer": {
                "par_factors": cls.par_factors,
            },
            "stream-vectorizer": {
                "stream-shape": cls.stream_shape,
                "enable-block-sparse": cls.block_sparse,
            },
            "dataflow-ordering": cls.dataflow_ordering,
            "sparse-annotations": cls.sparse_annotations,
        }

    @classmethod
    def missing_fields(cls):
        missing = []
        if cls.par_factors is None:
            missing.append("par_factors")
        if cls.stream_shape is None:
            missing.append("stream_shape")
        if cls.block_sparse is None:
            missing.append("block_sparse")
        if cls.dataflow_ordering is None:
            missing.append("dataflow_ordering")
        if cls.sparse_annotations is None:
            missing.append("sparse_annotations")
        return missing

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
    num_tensors: int
    """Number of tensors in the MLIR program"""

@Input
class TensorInfo:
    """A tensor in the MLIR program

    This tensor can be annotated with a sparse format or per-index sparsity."""
    name: str
    """Name of the tensor (e.g. t0)"""
    path: str
    """Path to the MLIR program this tensor belongs to"""
    tensor_order: int
    """Order index for sequential annotation (0-based)"""
    num_indices: int
    """Number of indices (dimensions) of this tensor"""
    indices: str
    """Space-separated names of the tensor's indices (e.g. 'i j')"""

@Input
class LoopOrderOption:
    """LoopOrderOption

    A loop ordering option generated by the FuseFlow compiler."""
    path: str
    """Path to the MLIR program this ordering applies to"""
    order: str
    """Loop order string"""

@Output
class TensorAnnotationLevel:
    """TensorAnnotationLevel

    Ready to annotate tensor T: pick a sparse format or enter per-index custom mode."""
    path: str

    tensor_index: int
    tensor_name: str
    num_indices: int
    indices: str
    mlir_path: str
    num_tensors: int
    num_loops: int


@Output
class TensorAnnotationDone:
    """TensorAnnotationDone

    Tensor T has been fully annotated (via a format shortcut or per-index custom)."""
    path: str

    tensor_index: int
    tensor_name: str
    mlir_path: str
    num_tensors: int
    num_loops: int


@Output
class TensorIndexLevel:
    """TensorIndexLevel

    Ready to annotate index K of tensor T (custom per-index mode)."""
    path: str

    tensor_index: int
    tensor_name: str
    num_indices: int
    indices: str
    index_position: int
    mlir_path: str
    num_tensors: int
    num_loops: int


@Output
class TensorIndexChoice:
    """TensorIndexChoice

    Index K of tensor T has been marked sparse or dense."""
    path: str

    tensor_index: int
    tensor_name: str
    num_indices: int
    indices: str
    index_position: int
    is_sparse: bool
    mlir_path: str
    num_tensors: int
    num_loops: int


@Output
class SparseAnnotationPass:
    """SparseAnnotationPass

    Completion of sparse annotation for all tensors in the MLIR program."""
    path: str

    mlir_path: str
    num_loops: int


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
class ParFactorLevelChoice:
    """ParFactorLevelChoice

    Internal step to enumerate stream levels for per-level parallelization factors."""
    path: str

    stream_level: int
    mlir_path: str
    num_loops: int


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
    stream_level: int
    mlir_path: str
    num_loops: int


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

@Function(
)
def default_schedule(__hb_ret: FuseFlowSchedule):
    """schedule 

    The function that produces a schedule."""
    ScheduleState.set_values(
        par_factors=[1],
        stream_shape=16,
        block_sparse=False,
        dataflow_ordering="",
    )
    schedule = ScheduleState.as_dict()
    write_schedule_json(schedule, __hb_ret.path)
    print(json.dumps(schedule, indent=2, sort_keys=True))


@Function(
)
def build_schedule(__hb_order: LoopOrderChoice, __hb_ret: FuseFlowSchedule):
    missing = ScheduleState.missing_fields()
    if missing:
        raise RuntimeError(
            "schedule state incomplete; missing: " + ", ".join(missing)
        )
    schedule = ScheduleState.as_dict()
    write_schedule_json(schedule, __hb_ret.path)
    print(json.dumps(schedule, indent=2, sort_keys=True))

################################################################################
# %% Sparse Tensor Annotation (format shortcuts + per-index custom mode)
#
# At each tensor, the user picks one of:
#   * a format shortcut (CSR, CSC, COO for 2D; Dense for any dim), which commits
#     every index of the tensor at once, or
#   * "custom", which drops into a per-index loop where each index position K
#     gets its own hole (t{N}_{index_name}) marked sparse or dense.
# Max indices per tensor: 4. Max tensors: 16.

# --- Enter annotation for tensor 0 ----------------------------------------

@Function(
    "ret.tensor_index = 0",
    "ret.tensor_index < mlir.num_tensors",
    "mlir.num_tensors < 17",
    "ret.num_tensors = mlir.num_tensors",
    "ret.num_loops = mlir.num_loops",
    "ret.mlir_path = mlir.path",
    "P_TensorInfo { path = mlir.path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def begin_tensor_annotation(__hb_mlir: MlirProgram, __hb_ret: TensorAnnotationLevel):
    print(
        f"Begin tensor annotation: '{__hb_ret.tensor_name}' "
        f"({__hb_ret.num_indices} indices: {__hb_ret.indices})."
    )

# --- Format shortcuts: Level -> Done --------------------------------------

@Function(
    "level.num_indices = 2",
    "ret.tensor_index = level.tensor_index",
    "ret.tensor_name = level.tensor_name",
    "ret.num_tensors = level.num_tensors",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def annotate_tensor_csr(__hb_level: TensorAnnotationLevel, __hb_ret: TensorAnnotationDone):
    names = __hb_level.indices.split()
    ScheduleState.annotate_tensor_format(__hb_level.tensor_name, "CSR", names, [True, False])
    print(f"Annotated '{__hb_level.tensor_name}' as CSR: {names[0]}=sparse, {names[1]}=dense.")

@Function(
    "level.num_indices = 2",
    "ret.tensor_index = level.tensor_index",
    "ret.tensor_name = level.tensor_name",
    "ret.num_tensors = level.num_tensors",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def annotate_tensor_csc(__hb_level: TensorAnnotationLevel, __hb_ret: TensorAnnotationDone):
    names = __hb_level.indices.split()
    ScheduleState.annotate_tensor_format(__hb_level.tensor_name, "CSC", names, [False, True])
    print(f"Annotated '{__hb_level.tensor_name}' as CSC: {names[0]}=dense, {names[1]}=sparse.")

@Function(
    "level.num_indices = 2",
    "ret.tensor_index = level.tensor_index",
    "ret.tensor_name = level.tensor_name",
    "ret.num_tensors = level.num_tensors",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def annotate_tensor_coo(__hb_level: TensorAnnotationLevel, __hb_ret: TensorAnnotationDone):
    names = __hb_level.indices.split()
    ScheduleState.annotate_tensor_format(__hb_level.tensor_name, "COO", names, [True, True])
    print(f"Annotated '{__hb_level.tensor_name}' as COO: {names[0]},{names[1]} both sparse.")

@Function(
    "ret.tensor_index = level.tensor_index",
    "ret.tensor_name = level.tensor_name",
    "ret.num_tensors = level.num_tensors",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def annotate_tensor_dense(__hb_level: TensorAnnotationLevel, __hb_ret: TensorAnnotationDone):
    names = __hb_level.indices.split()
    ScheduleState.annotate_tensor_format(
        __hb_level.tensor_name, "Dense", names, [False] * len(names)
    )
    print(f"Annotated '{__hb_level.tensor_name}' as Dense ({len(names)}D).")

# --- Custom per-index mode: Level -> IndexLevel ---------------------------

@Function(
    "ret.tensor_index = level.tensor_index",
    "ret.tensor_name = level.tensor_name",
    "ret.num_indices = level.num_indices",
    "ret.indices = level.indices",
    "ret.index_position = 0",
    "ret.index_position < level.num_indices",
    "level.num_indices < 5",
    "ret.num_tensors = level.num_tensors",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def begin_custom_annotation(__hb_level: TensorAnnotationLevel, __hb_ret: TensorIndexLevel):
    names = __hb_level.indices.split()
    first = names[0] if names else "?"
    print(
        f"Begin custom per-index annotation of '{__hb_level.tensor_name}' "
        f"at hole {__hb_level.tensor_name}_{first}."
    )

# --- Per-index choice: IndexLevel -> IndexChoice --------------------------

@Function(
    "ret.is_sparse = true",
    "ret.tensor_index = il.tensor_index",
    "ret.tensor_name = il.tensor_name",
    "ret.num_indices = il.num_indices",
    "ret.indices = il.indices",
    "ret.index_position = il.index_position",
    "ret.num_tensors = il.num_tensors",
    "ret.num_loops = il.num_loops",
    "ret.mlir_path = il.mlir_path",
)
def annotate_index_sparse(__hb_il: TensorIndexLevel, __hb_ret: TensorIndexChoice):
    names = __hb_il.indices.split()
    idx = names[__hb_il.index_position]
    ScheduleState.annotate_tensor_index(__hb_il.tensor_name, idx, True)
    print(f"Annotated {__hb_il.tensor_name}_{idx} as sparse.")

@Function(
    "ret.is_sparse = false",
    "ret.tensor_index = il.tensor_index",
    "ret.tensor_name = il.tensor_name",
    "ret.num_indices = il.num_indices",
    "ret.indices = il.indices",
    "ret.index_position = il.index_position",
    "ret.num_tensors = il.num_tensors",
    "ret.num_loops = il.num_loops",
    "ret.mlir_path = il.mlir_path",
)
def annotate_index_dense(__hb_il: TensorIndexLevel, __hb_ret: TensorIndexChoice):
    names = __hb_il.indices.split()
    idx = names[__hb_il.index_position]
    ScheduleState.annotate_tensor_index(__hb_il.tensor_name, idx, False)
    print(f"Annotated {__hb_il.tensor_name}_{idx} as dense.")

# --- Advance to next index (custom): IndexChoice -> IndexLevel ------------

@Function(
    "prev.index_position = 0",
    "ret.index_position = 1",
    "ret.index_position < prev.num_indices",
    "ret.tensor_index = prev.tensor_index",
    "ret.tensor_name = prev.tensor_name",
    "ret.num_indices = prev.num_indices",
    "ret.indices = prev.indices",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_index_1(__hb_prev: TensorIndexChoice, __hb_ret: TensorIndexLevel):
    print("Advance to index position 1.")

@Function(
    "prev.index_position = 1",
    "ret.index_position = 2",
    "ret.index_position < prev.num_indices",
    "ret.tensor_index = prev.tensor_index",
    "ret.tensor_name = prev.tensor_name",
    "ret.num_indices = prev.num_indices",
    "ret.indices = prev.indices",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_index_2(__hb_prev: TensorIndexChoice, __hb_ret: TensorIndexLevel):
    print("Advance to index position 2.")

@Function(
    "prev.index_position = 2",
    "ret.index_position = 3",
    "ret.index_position < prev.num_indices",
    "ret.tensor_index = prev.tensor_index",
    "ret.tensor_name = prev.tensor_name",
    "ret.num_indices = prev.num_indices",
    "ret.indices = prev.indices",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_index_3(__hb_prev: TensorIndexChoice, __hb_ret: TensorIndexLevel):
    print("Advance to index position 3.")

# --- Finish custom when at last index: IndexChoice -> Done ----------------

@Function(
    "prev.index_position = 0",
    "prev.num_indices = 1",
    "ret.tensor_index = prev.tensor_index",
    "ret.tensor_name = prev.tensor_name",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def finish_custom_after_index_0(__hb_prev: TensorIndexChoice, __hb_ret: TensorAnnotationDone):
    print(f"Finished custom annotation of '{__hb_prev.tensor_name}'.")

@Function(
    "prev.index_position = 1",
    "prev.num_indices = 2",
    "ret.tensor_index = prev.tensor_index",
    "ret.tensor_name = prev.tensor_name",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def finish_custom_after_index_1(__hb_prev: TensorIndexChoice, __hb_ret: TensorAnnotationDone):
    print(f"Finished custom annotation of '{__hb_prev.tensor_name}'.")

@Function(
    "prev.index_position = 2",
    "prev.num_indices = 3",
    "ret.tensor_index = prev.tensor_index",
    "ret.tensor_name = prev.tensor_name",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def finish_custom_after_index_2(__hb_prev: TensorIndexChoice, __hb_ret: TensorAnnotationDone):
    print(f"Finished custom annotation of '{__hb_prev.tensor_name}'.")

@Function(
    "prev.index_position = 3",
    "prev.num_indices = 4",
    "ret.tensor_index = prev.tensor_index",
    "ret.tensor_name = prev.tensor_name",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def finish_custom_after_index_3(__hb_prev: TensorIndexChoice, __hb_ret: TensorAnnotationDone):
    print(f"Finished custom annotation of '{__hb_prev.tensor_name}'.")

# --- Advance to next tensor: Done -> Level --------------------------------

@Function(
    "prev.tensor_index = 0",
    "ret.tensor_index = 1",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_1(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 1.")

@Function(
    "prev.tensor_index = 1",
    "ret.tensor_index = 2",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_2(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 2.")

@Function(
    "prev.tensor_index = 2",
    "ret.tensor_index = 3",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_3(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 3.")

@Function(
    "prev.tensor_index = 3",
    "ret.tensor_index = 4",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_4(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 4.")

@Function(
    "prev.tensor_index = 4",
    "ret.tensor_index = 5",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_5(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 5.")

@Function(
    "prev.tensor_index = 5",
    "ret.tensor_index = 6",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_6(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 6.")

@Function(
    "prev.tensor_index = 6",
    "ret.tensor_index = 7",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_7(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 7.")

@Function(
    "prev.tensor_index = 7",
    "ret.tensor_index = 8",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_8(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 8.")

@Function(
    "prev.tensor_index = 8",
    "ret.tensor_index = 9",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_9(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 9.")

@Function(
    "prev.tensor_index = 9",
    "ret.tensor_index = 10",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_10(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 10.")

@Function(
    "prev.tensor_index = 10",
    "ret.tensor_index = 11",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_11(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 11.")

@Function(
    "prev.tensor_index = 11",
    "ret.tensor_index = 12",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_12(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 12.")

@Function(
    "prev.tensor_index = 12",
    "ret.tensor_index = 13",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_13(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 13.")

@Function(
    "prev.tensor_index = 13",
    "ret.tensor_index = 14",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_14(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 14.")

@Function(
    "prev.tensor_index = 14",
    "ret.tensor_index = 15",
    "ret.tensor_index < prev.num_tensors",
    "ret.num_tensors = prev.num_tensors",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
    "P_TensorInfo { path = prev.mlir_path, name = ret.tensor_name, tensor_order = ret.tensor_index, num_indices = ret.num_indices, indices = ret.indices }",
)
def advance_tensor_annotation_15(__hb_prev: TensorAnnotationDone, __hb_ret: TensorAnnotationLevel):
    print("Advance to tensor annotation index 15.")

# --- Finish sparse-annotation pass: Done -> SparseAnnotationPass ----------

@Function(
    "done.tensor_index = 0",
    "done.num_tensors = 1",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_0(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 1",
    "done.num_tensors = 2",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_1(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 2",
    "done.num_tensors = 3",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_2(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 3",
    "done.num_tensors = 4",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_3(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 4",
    "done.num_tensors = 5",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_4(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 5",
    "done.num_tensors = 6",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_5(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 6",
    "done.num_tensors = 7",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_6(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 7",
    "done.num_tensors = 8",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_7(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 8",
    "done.num_tensors = 9",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_8(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 9",
    "done.num_tensors = 10",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_9(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 10",
    "done.num_tensors = 11",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_10(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 11",
    "done.num_tensors = 12",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_11(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 12",
    "done.num_tensors = 13",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_12(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 13",
    "done.num_tensors = 14",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_13(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 14",
    "done.num_tensors = 15",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_14(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

@Function(
    "done.tensor_index = 15",
    "done.num_tensors = 16",
    "ret.mlir_path = done.mlir_path",
    "ret.num_loops = done.num_loops",
)
def finish_annotation_after_tensor_15(__hb_done: TensorAnnotationDone, __hb_ret: SparseAnnotationPass):
    print("Sparse annotation complete.")

################################################################################
# %% Parallelization Factor Selection

@Function(
    "ret.stream_level = 0",
    "ret.stream_level < vec.num_loops",
    "vec.num_loops < 17",
    "ret.num_loops = vec.num_loops",
    "ret.mlir_path = vec.mlir_path",
)
def begin_par_factor_selection(__hb_vec: VectorizationPass, __hb_ret: ParFactorLevelChoice):
    ScheduleState.initialize_parallel_schedule(num_loops=__hb_vec.num_loops, max_levels=16)
    print("Begin per-level par factor selection at stream level 0.")

@Function(
    "prev.stream_level = 0",
    "ret.stream_level = 1",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_1(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 1.")

@Function(
    "prev.stream_level = 1",
    "ret.stream_level = 2",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_2(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 2.")

@Function(
    "prev.stream_level = 2",
    "ret.stream_level = 3",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_3(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 3.")

@Function(
    "prev.stream_level = 3",
    "ret.stream_level = 4",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_4(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 4.")

@Function(
    "prev.stream_level = 4",
    "ret.stream_level = 5",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_5(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 5.")

@Function(
    "prev.stream_level = 5",
    "ret.stream_level = 6",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_6(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 6.")

@Function(
    "prev.stream_level = 6",
    "ret.stream_level = 7",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_7(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 7.")

@Function(
    "prev.stream_level = 7",
    "ret.stream_level = 8",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_8(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 8.")

@Function(
    "prev.stream_level = 8",
    "ret.stream_level = 9",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_9(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 9.")

@Function(
    "prev.stream_level = 9",
    "ret.stream_level = 10",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_10(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 10.")

@Function(
    "prev.stream_level = 10",
    "ret.stream_level = 11",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_11(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 11.")

@Function(
    "prev.stream_level = 11",
    "ret.stream_level = 12",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_12(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 12.")

@Function(
    "prev.stream_level = 12",
    "ret.stream_level = 13",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_13(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 13.")

@Function(
    "prev.stream_level = 13",
    "ret.stream_level = 14",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_14(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 14.")

@Function(
    "prev.stream_level = 14",
    "ret.stream_level = 15",
    "ret.stream_level < prev.num_loops",
    "ret.num_loops = prev.num_loops",
    "ret.mlir_path = prev.mlir_path",
)
def advance_par_factor_level_15(__hb_prev: ParFactorChoice, __hb_ret: ParFactorLevelChoice):
    print("Advance to stream level 15.")

@Function(
    "ret.par_factor = 1",
    "ret.stream_level = level.stream_level",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def choose_default_par_factor(__hb_level: ParFactorLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_par_factor_for_level(
        stream_level=__hb_level.stream_level,
        par_factor=1,
    )
    print("Choose default par factor (1).")

@Function(
    "ret.par_factor = 2",
    "ret.stream_level = level.stream_level",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_2(__hb_level: ParFactorLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_par_factor_for_level(
        stream_level=__hb_level.stream_level,
        par_factor=2,
    )
    print("Choose par factor 2.")

@Function(
    "ret.par_factor = 4",
    "ret.stream_level = level.stream_level",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_4(__hb_level: ParFactorLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_par_factor_for_level(
        stream_level=__hb_level.stream_level,
        par_factor=4,
    )
    print("Choose par factor 4.")

@Function(
    "ret.par_factor = 8",
    "ret.stream_level = level.stream_level",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_8(__hb_level: ParFactorLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_par_factor_for_level(
        stream_level=__hb_level.stream_level,
        par_factor=8,
    )
    print("Choose par factor 8.")

@Function(
    "ret.par_factor = 16",
    "ret.stream_level = level.stream_level",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_16(__hb_level: ParFactorLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_par_factor_for_level(
        stream_level=__hb_level.stream_level,
        par_factor=16,
    )
    print("Choose par factor 16.")

@Function(
    "ret.par_factor = 32",
    "ret.stream_level = level.stream_level",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_32(__hb_level: ParFactorLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_par_factor_for_level(
        stream_level=__hb_level.stream_level,
        par_factor=32,
    )
    print("Choose par factor 32.")

@Function(
    "ret.par_factor = 64",
    "ret.stream_level = level.stream_level",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_64(__hb_level: ParFactorLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_par_factor_for_level(
        stream_level=__hb_level.stream_level,
        par_factor=64,
    )
    print("Choose par factor 64.")

@Function(
    "ret.par_factor = 128",
    "ret.stream_level = level.stream_level",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_128(__hb_level: ParFactorLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_par_factor_for_level(
        stream_level=__hb_level.stream_level,
        par_factor=128,
    )
    print("Choose par factor 128.")

@Function(
    "ret.par_factor = 256",
    "ret.stream_level = level.stream_level",
    "ret.num_loops = level.num_loops",
    "ret.mlir_path = level.mlir_path",
)
def choose_par_factor_256(__hb_level: ParFactorLevelChoice, __hb_ret: ParFactorChoice):
    ScheduleState.set_par_factor_for_level(
        stream_level=__hb_level.stream_level,
        par_factor=256,
    )
    print("Choose par factor 256.")

@Function(
    "ret.stream_shape = 16",
    "ret.mlir_path = ann.mlir_path",
    "ret.num_loops = ann.num_loops",
)
def choose_default_stream_shape(__hb_ann: SparseAnnotationPass, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=16)
    print("Choose default stream shape (16).")

@Function(
    "ret.stream_shape = 1",
    "ret.mlir_path = ann.mlir_path",
    "ret.num_loops = ann.num_loops",
)
def choose_stream_shape_1(__hb_ann: SparseAnnotationPass, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=1)
    print("Choose stream shape 1.")

@Function(
    "ret.stream_shape = 2",
    "ret.mlir_path = ann.mlir_path",
    "ret.num_loops = ann.num_loops",
)
def choose_stream_shape_2(__hb_ann: SparseAnnotationPass, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=2)
    print("Choose stream shape 2.")

@Function(
    "ret.stream_shape = 4",
    "ret.mlir_path = ann.mlir_path",
    "ret.num_loops = ann.num_loops",
)
def choose_stream_shape_4(__hb_ann: SparseAnnotationPass, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=4)
    print("Choose stream shape 4.")

@Function(
    "ret.stream_shape = 8",
    "ret.mlir_path = ann.mlir_path",
    "ret.num_loops = ann.num_loops",
)
def choose_stream_shape_8(__hb_ann: SparseAnnotationPass, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=8)
    print("Choose stream shape 8.")

@Function(
    "ret.stream_shape = 16",
    "ret.mlir_path = ann.mlir_path",
    "ret.num_loops = ann.num_loops",
)
def choose_stream_shape_16(__hb_ann: SparseAnnotationPass, __hb_ret: StreamShapeChoice):
    ScheduleState.set_values(stream_shape=16)
    print("Choose stream shape 16.")

@Function(
    "ret.block_sparse = false",
    "ret.mlir_path = shape.mlir_path",
    "ret.num_loops = shape.num_loops",
)
def choose_default_block_sparse(__hb_shape: StreamShapeChoice, __hb_ret: BlockSparseChoice):
    ScheduleState.set_values(block_sparse=False)
    print("Choose default block sparse (false).")

@Function(
    "ret.block_sparse = true",
    "ret.mlir_path = shape.mlir_path",
    "ret.num_loops = shape.num_loops",
)
def choose_block_sparse_true(__hb_shape: StreamShapeChoice, __hb_ret: BlockSparseChoice):
    ScheduleState.set_values(block_sparse=True)
    print("Choose block sparse (true).")

@Function(
    "ret.mlir_path = block.mlir_path",
    "ret.num_loops = block.num_loops",
)
def vectorization(__hb_block: BlockSparseChoice, __hb_ret: VectorizationPass):
    print("Vectorization.")

@Function(
    "par.stream_level = 0",
    "par.num_loops = 1",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_0(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 1",
    "par.num_loops = 2",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_1(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 2",
    "par.num_loops = 3",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_2(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 3",
    "par.num_loops = 4",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_3(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 4",
    "par.num_loops = 5",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_4(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 5",
    "par.num_loops = 6",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_5(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 6",
    "par.num_loops = 7",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_6(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 7",
    "par.num_loops = 8",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_7(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 8",
    "par.num_loops = 9",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_8(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 9",
    "par.num_loops = 10",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_9(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 10",
    "par.num_loops = 11",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_10(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 11",
    "par.num_loops = 12",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_11(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 12",
    "par.num_loops = 13",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_12(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 13",
    "par.num_loops = 14",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_13(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 14",
    "par.num_loops = 15",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_14(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "par.stream_level = 15",
    "par.num_loops = 16",
    "ret.mlir_path = par.mlir_path",
)
def parallelization_after_level_15(__hb_par: ParFactorChoice, __hb_ret: ParallelizationPass):
    print("Parallelization.")

@Function(
    "P_LoopOrderOption { path = pass.mlir_path, order = ret.order }",
)
def choose_loop_order(__hb_pass: ParallelizationPass, __hb_ret: LoopOrderChoice):
    ScheduleState.set_values(dataflow_ordering=__hb_ret.order)
    print("Choose dataflow loop order.")
