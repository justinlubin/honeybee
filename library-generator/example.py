from dataclasses import dataclass

from lib import Function, Prop, Type


@Prop
@dataclass
class RNAseq:
    "RNA-sequencing data"

    sample: str
    "A label for the sample (you can choose anything you'd like)"

    path: str
    "The path to the folder containing the raw FASTQ files for the sample"


@Type(suggested_variable_name="rnaseq_data")
@dataclass
class LoadedRNAseq:
    "Loaded RNA-sequencing data"

    sample: str
    path: str


@Type(suggested_variable_name="read_counts")
@dataclass
class ReadCountVector:
    "A vector of read counts"

    sample: str


@Function(
    "ret.sample = data.sample",
)
def kallisto(data: LoadedRNAseq) -> ReadCountVector:
    "The kallisto RNAseq quantifier"
    print("Do stuff here!")
