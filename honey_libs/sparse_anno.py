import json
import os

from honey_lang import Helper, Input, Output, Function, initialize

initialize()


@Helper
def write_annotation_json(state_dict, output_dir, *, filename="annotation.json"):
    os.makedirs(output_dir, exist_ok=True)
    out_path = os.path.join(output_dir, filename)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(state_dict, f, indent=2, sort_keys=True)
    return out_path


@Helper
class SparseState:
    """Per-session sparse annotation state for a single tensor.

    Reset at the start of every annotation (begin_annotation / shortcut entry),
    mutated through the chosen path, and serialized to JSON when the goal is
    reached.
    """

    tensor_name = None
    is_sparse = None
    dimensions = None  # dict: index_name -> bool

    @classmethod
    def reset(cls, tensor_name, index_names):
        cls.tensor_name = tensor_name
        cls.is_sparse = None
        cls.dimensions = {name: False for name in index_names}

    @classmethod
    def set_format(cls, *, is_sparse, sparsity_pattern, index_names):
        cls.is_sparse = is_sparse
        for name, sparse in zip(index_names, sparsity_pattern):
            cls.dimensions[name] = sparse

    @classmethod
    def set_index(cls, index_name, is_sparse):
        cls.dimensions[index_name] = is_sparse

    @classmethod
    def as_dict(cls):
        return {
            "tensor_name": cls.tensor_name,
            "sparse": bool(cls.is_sparse),
            "dimensions": dict(cls.dimensions or {}),
        }

    @classmethod
    def write(cls, output_dir):
        return write_annotation_json(cls.as_dict(), output_dir)


# --------------------------------------------------------------------------
# Inputs
# --------------------------------------------------------------------------

@Input
class Tensor:
    """A tensor whose sparsity we want to annotate."""
    name: str
    """Name of the tensor (e.g. t0)."""
    num_indices: int
    """Number of indices (dimensions) of this tensor."""
    indices: str
    """Space-separated names of the tensor's indices (e.g. 'i0 i1')."""


# --------------------------------------------------------------------------
# Intermediate outputs
# --------------------------------------------------------------------------

@Output
class TensorAnnotationLevel:
    """@intermediate:Entry point for annotating a single tensor."""
    path: str
    tensor_name: str
    num_indices: int
    indices: str


@Output
class TensorIndexLevel:
    """@intermediate:Annotating a specific index of a tensor (custom mode)."""
    path: str
    tensor_name: str
    num_indices: int
    indices: str
    index_position: int


@Output
class TensorIndexChoice:
    """@intermediate:An index has been annotated as sparse or dense."""
    path: str
    tensor_name: str
    num_indices: int
    indices: str
    index_position: int
    is_sparse: bool


# --------------------------------------------------------------------------
# Goal
# --------------------------------------------------------------------------

@Output
class TensorSparseAnnotation:
    """The sparse annotation for a single tensor.

    The goal of this step is to decide whether the tensor is sparse, and if so
    which of its dimensions are compressed. The result is written as JSON to
    the output path."""
    path: str


# --------------------------------------------------------------------------
# Begin: Tensor -> Level
# --------------------------------------------------------------------------

@Function(
    "ret.tensor_name = tensor.name",
    "ret.num_indices = tensor.num_indices",
    "ret.indices = tensor.indices",
)
def begin_annotation(__hb_tensor: Tensor, __hb_ret: TensorAnnotationLevel):
    SparseState.reset(__hb_tensor.name, __hb_tensor.indices.split())
    print(
        f"Begin sparse annotation: '{__hb_tensor.name}' "
        f"({__hb_tensor.num_indices} indices: {__hb_tensor.indices})."
    )


# --------------------------------------------------------------------------
# Format shortcuts: Level -> Goal
# --------------------------------------------------------------------------

@Function()
def annotate_dense(__hb_level: TensorAnnotationLevel, __hb_ret: TensorSparseAnnotation):
    names = __hb_level.indices.split()
    SparseState.reset(__hb_level.tensor_name, names)
    SparseState.set_format(
        is_sparse=False,
        sparsity_pattern=[False] * len(names),
        index_names=names,
    )
    SparseState.write(__hb_ret.path)
    print(f"Annotated '{__hb_level.tensor_name}' as Dense ({len(names)}D).")


@Function(
    "level.num_indices = 2",
)
def annotate_csr(__hb_level: TensorAnnotationLevel, __hb_ret: TensorSparseAnnotation):
    names = __hb_level.indices.split()
    SparseState.reset(__hb_level.tensor_name, names)
    SparseState.set_format(
        is_sparse=True,
        sparsity_pattern=[False, True],
        index_names=names,
    )
    SparseState.write(__hb_ret.path)
    print(f"Annotated '{__hb_level.tensor_name}' as CSR: {names[0]}=dense, {names[1]}=sparse.")


@Function(
    "level.num_indices = 2",
)
def annotate_csc(__hb_level: TensorAnnotationLevel, __hb_ret: TensorSparseAnnotation):
    names = __hb_level.indices.split()
    SparseState.reset(__hb_level.tensor_name, names)
    SparseState.set_format(
        is_sparse=True,
        sparsity_pattern=[True, False],
        index_names=names,
    )
    SparseState.write(__hb_ret.path)
    print(f"Annotated '{__hb_level.tensor_name}' as CSC: {names[0]}=sparse, {names[1]}=dense.")


@Function(
    "level.num_indices = 2",
)
def annotate_dcsr(__hb_level: TensorAnnotationLevel, __hb_ret: TensorSparseAnnotation):
    names = __hb_level.indices.split()
    SparseState.reset(__hb_level.tensor_name, names)
    SparseState.set_format(
        is_sparse=True,
        sparsity_pattern=[True, True],
        index_names=names,
    )
    SparseState.write(__hb_ret.path)
    print(f"Annotated '{__hb_level.tensor_name}' as DCSR: {names[0]}=sparse, {names[1]}=sparse.")


@Function(
    "1 < level.num_indices",
)
def annotate_coo(__hb_level: TensorAnnotationLevel, __hb_ret: TensorSparseAnnotation):
    names = __hb_level.indices.split()
    SparseState.reset(__hb_level.tensor_name, names)
    SparseState.set_format(
        is_sparse=True,
        sparsity_pattern=[True] * len(names),
        index_names=names,
    )
    SparseState.write(__hb_ret.path)
    print(f"Annotated '{__hb_level.tensor_name}' as COO ({len(names)}D, all sparse).")


# --------------------------------------------------------------------------
# Custom per-index mode: Level -> IndexLevel
# --------------------------------------------------------------------------

@Function(
    "ret.tensor_name = level.tensor_name",
    "ret.num_indices = level.num_indices",
    "ret.indices = level.indices",
    "ret.index_position = 0",
    "ret.index_position < level.num_indices",
    "level.num_indices < 5",
)
def begin_custom_annotation(__hb_level: TensorAnnotationLevel, __hb_ret: TensorIndexLevel):
    names = __hb_level.indices.split()
    SparseState.reset(__hb_level.tensor_name, names)
    SparseState.is_sparse = True
    first = names[0] if names else "?"
    print(
        f"Begin custom per-index annotation of '{__hb_level.tensor_name}' "
        f"at hole {__hb_level.tensor_name}_{first}."
    )


# --------------------------------------------------------------------------
# Per-index choice: IndexLevel -> IndexChoice
# --------------------------------------------------------------------------

@Function(
    "ret.is_sparse = true",
    "ret.tensor_name = il.tensor_name",
    "ret.num_indices = il.num_indices",
    "ret.indices = il.indices",
    "ret.index_position = il.index_position",
)
def annotate_index_sparse(__hb_il: TensorIndexLevel, __hb_ret: TensorIndexChoice):
    names = __hb_il.indices.split()
    idx = names[__hb_il.index_position]
    SparseState.set_index(idx, True)
    print(f"Annotated {__hb_il.tensor_name}_{idx} as sparse.")


@Function(
    "ret.is_sparse = false",
    "ret.tensor_name = il.tensor_name",
    "ret.num_indices = il.num_indices",
    "ret.indices = il.indices",
    "ret.index_position = il.index_position",
)
def annotate_index_dense(__hb_il: TensorIndexLevel, __hb_ret: TensorIndexChoice):
    names = __hb_il.indices.split()
    idx = names[__hb_il.index_position]
    SparseState.set_index(idx, False)
    print(f"Annotated {__hb_il.tensor_name}_{idx} as dense.")


# --------------------------------------------------------------------------
# Advance to next index (custom): IndexChoice -> IndexLevel
# --------------------------------------------------------------------------

@Function(
    "prev.index_position = 0",
    "ret.index_position = 1",
    "ret.index_position < prev.num_indices",
    "ret.tensor_name = prev.tensor_name",
    "ret.num_indices = prev.num_indices",
    "ret.indices = prev.indices",
)
def advance_index_1(__hb_prev: TensorIndexChoice, __hb_ret: TensorIndexLevel):
    print("Advance to index position 1.")


@Function(
    "prev.index_position = 1",
    "ret.index_position = 2",
    "ret.index_position < prev.num_indices",
    "ret.tensor_name = prev.tensor_name",
    "ret.num_indices = prev.num_indices",
    "ret.indices = prev.indices",
)
def advance_index_2(__hb_prev: TensorIndexChoice, __hb_ret: TensorIndexLevel):
    print("Advance to index position 2.")


@Function(
    "prev.index_position = 2",
    "ret.index_position = 3",
    "ret.index_position < prev.num_indices",
    "ret.tensor_name = prev.tensor_name",
    "ret.num_indices = prev.num_indices",
    "ret.indices = prev.indices",
)
def advance_index_3(__hb_prev: TensorIndexChoice, __hb_ret: TensorIndexLevel):
    print("Advance to index position 3.")


# --------------------------------------------------------------------------
# Finish custom when at last index: IndexChoice -> Goal
# --------------------------------------------------------------------------

@Function(
    "prev.index_position = 0",
    "prev.num_indices = 1",
)
def finish_custom_after_index_0(__hb_prev: TensorIndexChoice, __hb_ret: TensorSparseAnnotation):
    SparseState.write(__hb_ret.path)
    print(f"Finished custom annotation of '{__hb_prev.tensor_name}'.")


@Function(
    "prev.index_position = 1",
    "prev.num_indices = 2",
)
def finish_custom_after_index_1(__hb_prev: TensorIndexChoice, __hb_ret: TensorSparseAnnotation):
    SparseState.write(__hb_ret.path)
    print(f"Finished custom annotation of '{__hb_prev.tensor_name}'.")


@Function(
    "prev.index_position = 2",
    "prev.num_indices = 3",
)
def finish_custom_after_index_2(__hb_prev: TensorIndexChoice, __hb_ret: TensorSparseAnnotation):
    SparseState.write(__hb_ret.path)
    print(f"Finished custom annotation of '{__hb_prev.tensor_name}'.")


@Function(
    "prev.index_position = 3",
    "prev.num_indices = 4",
)
def finish_custom_after_index_3(__hb_prev: TensorIndexChoice, __hb_ret: TensorSparseAnnotation):
    SparseState.write(__hb_ret.path)
    print(f"Finished custom annotation of '{__hb_prev.tensor_name}'.")
