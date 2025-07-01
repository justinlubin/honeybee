from dataclasses import dataclass

from lib import Function, Helper, Prop, Type


@Helper
def RUN(command):
    import subprocess

    return subprocess.run(
        command,
        shell=True,
        capture_output=True,
        text=True,
    ).stdout.split()


@Helper
def sample_names(filename):
    sn = RUN(f"cat {filename} | cut -d ',' -f1 | tail -n +2")
    return sn


################################################################################
# RNA-seq


# TODO auto-generate?
@Prop
class RNASeqProp:
    "RNA-seq"

    label: str
    "Label for data"

    sample_sheet: str
    "Path to sample sheet CSV"

    raw_data: str
    "Path to raw FASTQ files"


@Type(var_name="RNA_SAMPLES")
class RNASeqSamples:
    "Loaded RNA-seq sample metadata"

    @dataclass
    class S:
        label: str
        "Label for data"

        sample_sheet: str
        "Path to sample sheet CSV"

        raw_data: str
        "Path to raw FASTQ files"

    @dataclass
    class D:
        pass


# TODO auto-generate?
@Function(
    "RNASeqProp { label = ret.label, sample_sheet = ret.sample_sheet, raw_data = ret.raw_data }"
)
def get_rna_seq_samples(ret: RNASeqSamples.S) -> RNASeqSamples.D:
    """Load RNA-seq sample sheet"""
    return RNASeqSamples.D()


@Type
class RNASeq:
    "Loaded RNA-seq data"

    @dataclass
    class S:
        label: str
        "Label for data"

        qc: bool
        "Whether or not quality checks have been run"

    @dataclass
    class D:
        sample_sheet: str
        path: str


@Function(
    "ret.label = samples.label",
    "ret.qc = false",
)
def load_rna_seq(samples: RNASeqSamples, ret: RNASeq.S) -> RNASeq.D:
    """Directly load RNA-seq dataset"""
    return RNASeq.D(
        sample_sheet=samples.static.sample_sheet,
        path=samples.static.raw_data,
    )


@Type
class TranscriptMatrices:
    "Read count (and TPM abundance) matrix for samples"

    @dataclass
    class S:
        label: str
        "Label for RNA-seq data to analyze"

    @dataclass
    class D:
        sample_sheet: str
        path: str


@Function(
    "data.qc = false",
    "ret.label = data.label",
    "ret.qc = true",
)
def fastqc(data: RNASeq, ret: RNASeq.S) -> RNASeq.D:
    """Quality-check sequencing data with FastQC"""

    print("Running fastqc...")

    label = data.static.label
    in_path = data.dynamic.path

    RUN(f"mkdir -p output/{label}/fastqc")
    RUN(f"fastqc -t 8 -o output/{label}/fastqc {in_path}/*.fastq*")
    RUN(f"multiqc --filename output/{label}/multiqc.html output/{label}/fastqc")

    return data.dynamic


@Function(
    "data.qc = false",
    "ret.label = data.label",
    "ret.qc = true",
)
def multiqc(data: RNASeq, ret: RNASeq.S) -> RNASeq.D:
    """Quality-check sequencing data with MultiQC"""
    pass


@Function(
    "data.qc = true",
    "ret.label = data.label",
    "ret.qc = false",
)
def cutadapt_illumina(data: RNASeq, ret: RNASeq.S) -> RNASeq.D:
    """Remove Illumina universal adaptor and poly-A tails with cutadapt"""

    in_path = data.dynamic.path
    ret_path = f"output/{ret.label}/cutadapt_trimmed"

    RUN(f"mkdir -p {ret_path}")
    for name in sample_names(data.sample_sheet):
        print(f"Running cutadapt_illumina on {name}...")
        RUN(f"""cutadapt \\
                    --cores=0 \\
                    -m 1 \\
                    --poly-a \\
                    -a AGATCGGAAGAGCACACGTCTGAACTCCAGTCA \\
                    -A AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT \\
                    -o {ret_path}/{name}_R1.fastq.gz \\
                    -p {ret_path}/{name}_R2.fastq.gz \\
                    {in_path}/{name}_R1_001.fastq.gz \\
                    {in_path}/{name}_R2_001.fastq.gz""")

    return RNASeq.D(
        sample_sheet=data.dynamic.sample_sheet,
        path=ret_path,
    )


@Function(
    "data.qc = true",
    "ret.label = data.label",
)
def kallisto(data: RNASeq, ret: TranscriptMatrices.S) -> TranscriptMatrices.D:
    """Quantify transcript abundances with kallisto"""

    in_path = data.dynamic.path
    ret_path = f"output/{ret.label}/kallisto_quant"

    RUN(f"mkdir {ret_path}")
    for name in sample_names(data.sample_sheet):
        print(f"Running kallisto on {name}...")

        RUN(f"""kallisto quant \\
                    -t 8 \\
                    -i KALLISTO_INDEX \\
                    -o {ret_path}/{name} \\
                    {in_path}/{name}_R1.fastq.gz \\
                    {in_path}/{name}_R2.fastq.gz""")

    return TranscriptMatrices.D(
        sample_sheet=data.dynamic.sample_sheet,
        path=ret_path,
    )


@Function(
    "data.qc = true",
    "ret.label = data.label",
)
def salmon(data: RNASeq, ret: TranscriptMatrices.S) -> TranscriptMatrices.D:
    """Quantify transcript abundances with salmon"""
    pass


@Function(
    "ret.label = data.label",
)
def combat_seq(
    data: TranscriptMatrices, ret: TranscriptMatrices.S
) -> TranscriptMatrices.D:
    """Correct for batch effects with ComBat-seq"""
    pass


@Type
class Alignment:
    "Alignment to a reference genome"

    @dataclass
    class S:
        label: str
        "Label for data"

    @dataclass
    class D:
        sample_sheet: str
        path: str


@Function(
    "ret.label = data.label",
)
def featureCounts(data: Alignment, ret: TranscriptMatrices.S) -> TranscriptMatrices.D:
    """Summarize aligned reads with featureCounts"""
    pass


@Function(
    "data.qc = true",
    "ret.label = data.label",
)
def star(data: RNASeq, ret: Alignment.S) -> Alignment.D:
    """Align spliced transcripts to a reference with STAR"""
    pass


################################################################################
# Stubs to implement


@Prop
class CutAndRunProp:
    "CUT&RUN-seq"

    label: str
    "Label for data"

    sample_sheet: str
    "Path to sample sheet CSV"

    raw_data: str
    "Path to raw FASTQ files"


@Prop
class EMSeqProp:
    "EM-seq"

    label: str
    "Label for data"

    sample_sheet: str
    "Path to sample sheet CSV"

    raw_data: str
    "Path to raw FASTQ files"


@Prop
class FlowProp:
    "Flow cytometry"

    label: str
    "Label for data"

    sample_sheet: str
    "Path to sample sheet CSV"

    raw_data: str
    "Path to raw FCS files"


@Prop
class SortProp:
    "Sort cells with FACS"

    label: str
    "Label for data"


@Prop
class StainProp:
    "Stain cells with antibodies"

    label: str
    "Label for data"


@Prop
class TransfectProp:
    "Infect cells with CRISPR sgRNA guide library"

    label: str
    "Label for data"

    library: str
    "Path to the library file"
