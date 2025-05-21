import subprocess

from lib import Function, Prop, Type


def RUN(command):
    subprocess.run(
        command,
        shell=True,
        capture_output=True,
        text=True,
    ).stdout.split()


def sample_names(filename):
    sn = RUN(f"cat {filename} | cut -d ',' -f1 | tail -n +2")
    return sn


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
    "RNA-seq samples"

    class S:
        label: str
        "Label for data"

        sample_sheet: str
        "Path to sample sheet CSV"

        raw_data: str
        "Path to raw FASTQ files"

    class D:
        pass


# TODO auto-generate?
@Function(
    "RNASeqProp { label = ret.label, sample_sheet = ret.sample_sheet, raw_data = ret.raw_data }"
)
def get_rna_seq_samples(ret: RNASeqSamples.S) -> RNASeqSamples.D:
    return RNASeqSamples.D()


@Type
class RNASeq:
    "RNA-seq data"

    class S:
        label: str
        "Label for data"

        qc: bool
        "Whether or not quality checks have been run"

    class D:
        sample_sheet: str
        path: str


@Function(
    "ret.label = samples.label",
    "ret.qc = false",
)
def load_rna_seq(samples: RNASeqSamples, ret: RNASeq.S) -> RNASeq.D:
    """Load RNA-seq dataset"""
    return RNASeq.D(
        sample_sheet=samples.static.sample_sheet,
        path=samples.static.raw_data,
    )


@Type
class TranscriptMatrices:
    "Transcript-by-sample matrices of counts and abundance (TPM)"

    class S:
        label: str
        "Label for RNA-seq data to analyze"

    class D:
        sample_sheet: str
        path: str


@Function(
    "data.qc = false",
    "ret.label = data.label",
    "ret.qc = true",
)
def fastqc(data: RNASeq, ret: RNASeq.S) -> RNASeq.D:
    """Quality-check RNA-seq data (FastQC)"""

    print("Running fastqc...")

    label = data.static.label
    in_path = data.dynamic.path

    RUN(f"mkdir -p output/{label}/fastqc")
    RUN(f"fastqc -t 8 -o output/{label}/fastqc {in_path}/*.fastq*")
    RUN(f"multiqc --filename output/{label}/multiqc.html output/{label}/fastqc")

    return data.dynamic


@Function(
    "data.qc = true",
    "ret.label = data.label",
    "ret.qc = false",
)
def cutadapt_illumina(data: RNASeq, ret: RNASeq.S) -> RNASeq.D:
    """Remove Illumina universal adaptor and poly-A tails (cutadapt)"""

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
    """Quantify transcript abundances (kallisto)"""

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
