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


@Prop
class RNASeq:
    "RNA-seq"

    label: str
    "Label for data"

    sample_sheet: str
    "Path to sample sheet CSV"

    raw_data: str
    "Path to raw FASTQ files"


@Type(suggested_variable_name="rna_seq_data")
class RNASeqDataset:
    "Quality-checked RNA-seq data"

    label: str
    "Label for data"

    sample_sheet: str
    "Path to sample sheet CSV"

    path: str
    "Path to data ('^' represents output)"

    qc: bool
    "Whether or not quality checks have been run"

    def full_path(self):
        self.data_path.replace("^", f"output/{self.label}/")


@Function(
    "RNASeq { label = ret.label, sample_sheet = ret.sample_sheet, raw_data = ret.path }",
    "ret.qc = false",
)
def load_rna_seq(ret: RNASeqDataset):
    """Load RNA-seq dataset"""


@Type(suggested_variable_name="read_counts")
class TranscriptMatrices:
    "Transcript-by-sample matrices of counts and abundance (TPM)"

    label: str
    "Label for data"

    sample_sheet: str
    "Path to sample sheet CSV"

    path: str
    "Path to data ('^' represents output)"


@Function(
    "data.qc = false",
    "ret.label = data.label",
    "ret.sample_sheet = data.sample_sheet",
    "ret.path = data.path",
    "ret.qc = true",
)
def fastqc(data: RNASeqDataset, ret: RNASeqDataset):
    """Quality-check RNA-seq data (FastQC)"""

    print("Running fastqc...")

    RUN(f"mkdir -p output/{data.label}/fastqc")
    RUN(f"fastqc -t 8 -o output/{data.label}/fastqc {data.data_path}/*.fastq*")
    RUN(
        f"multiqc --filename output/{data.label}/multiqc.html output/{data.label}/fastqc"
    )


@Function(
    "data.qc = true",
    "ret.label = data.label",
    "ret.sample_sheet = data.sample_sheet",
    'ret.path = "^trimmed"',
    "ret.qc = false",
)
def cutadapt_illumina(data: RNASeqDataset, ret: RNASeqDataset):
    """Remove Illumina universal adaptor and poly-A tails (cutadapt)"""

    RUN(f"mkdir {ret.full_path()}")
    for name in sample_names(data.sample_sheet):
        print(f"Running cutadapt_illumina on {name}...")
        RUN(f"""cutadapt \\
                    --cores=0 \\
                    -m 1 \\
                    --poly-a \\
                    -a AGATCGGAAGAGCACACGTCTGAACTCCAGTCA \\
                    -A AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT \\
                    -o {ret.full_path()}/{name}_R1.fastq.gz \\
                    -p {ret.full_path()}/{name}_R2.fastq.gz \\
                    {data.full_path()}/{name}_R1_001.fastq.gz \\
                    {data.full_path()}/{name}_R2_001.fastq.gz""")


@Function(
    "ret.label = data.label",
    "ret.sample_sheet = data.sample_sheet",
    'ret.path = "^quant"',
)
def kallisto(data: RNASeqDataset, ret: TranscriptMatrices):
    """Quantify transcript abundances (kallisto)"""

    RUN(f"mkdir {ret.full_path()}")
    for name in sample_names(data.sample_sheet):
        print(f"Running kallisto on {name}...")

        RUN(f"""kallisto quant \\
                    -t 8 \\
                    -i KALLISTO_INDEX \\
                    -o {ret.full_path()}/{name} \\
                    {data.full_path()}/{name}_R1.fastq.gz \\
                    {data.full_path()}/{name}_R2.fastq.gz""")
