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
# Raw RNA-seq data (reads)


@Prop
class P_SraRnaSeq:
    "RNA-seq data from SRA"

    label: str
    "Label for data"

    sra_sample_sheet: str
    "Path to sample sheet CSV with SRA metadata"


@Type
class SraRnaSeq:
    class S:
        "RNA-seq data from SRA"

        label: str
        "Label for data"

        sra_sample_sheet: str
        "Path to sample sheet CSV with SRA metadata"

    class D:
        pass


@Function(
    "P_SraRnaSeq { label = ret.label, sample_sheet = ret.sample_sheet, raw_data = ret.raw_data }",
)
def sra_rna_seq(ret: SraRnaSeq.S) -> SraRnaSeq.D:
    "TODO"
    SraRnaSeq.D()


@Prop
class P_LocalRnaSeq:
    "RNA-seq"

    label: str
    "Label for data"

    sample_sheet: str
    "Path to sample sheet CSV"

    raw_data: str
    "Path to directory containing raw reads (FASTQ files)"


@Type
class LocalRnaSeq:
    "RNA-seq"

    class S:
        label: str
        "Label for data"

        sample_sheet: str
        "Path to sample sheet CSV"

        raw_data: str
        "Path to directory containing raw reads (FASTQ files)"

    class D:
        pass


@Function(
    "P_LocalRnaSeq { label = ret.label, sample_sheet = ret.sample_sheet, raw_data = ret.raw_data }",
)
def local_rna_seq(ret: LocalRnaSeq.S) -> LocalRnaSeq.D:
    "TODO"
    LocalRnaSeq.D()


@Type
class RNASeq:
    "Raw RNA-seq data (reads)"

    class S:
        label: str
        "Label for data"

        qc: bool
        "Whether or not quality checks have been run"

    class D:
        sample_sheet: str
        path: str


@Function(
    "LocalRnaSeq { label = ret.label, sample_sheet = _, raw_data = _ }",
    "ret.qc = false",
)
def from_local_rna_seq(local: LocalRnaSeq, ret: RNASeq.S) -> RNASeq.D:
    """Load local RNA-seq

    # Directly load local RNA-seq reads

    This function loads raw RNA-seq data that you already have on your computer,
    typically in the .fastq.gz file format."""

    return RNASeq.D(
        sample_sheet=local.static.sample_sheet,
        path=local.static.raw_data,
    )


@Function(
    "SraRnaSeq { label = ret.label, sra_sample_sheet = _ }",
    "ret.qc = false",
)
def from_local_rna_seq(sra: SraRnaSeq, ret: RNASeq.S) -> RNASeq.D:
    """Load RNA-seq from SRA

    # Load RNA-seq reads from SRA database

    This function loads RNA-seq data from the
    [NCBI SRA database](https://www.ncbi.nlm.nih.gov/sra/)."""

    return RNASeq.D(
        sample_sheet=sra.static.sample_sheet,
        path=local.static.raw_data,
    )


################################################################################
# Raw RNA-seq data (reads)


@Type
class TranscriptMatrices:
    """Read count (and TPM abundance) matrix for RNA-seq samples

    The goal of this step is to calculate two two transcript × sample matrices:
    - One with (estimated) read counts.
    - One with TPM (transcripts-per-million) abundance.

    These matrices can be used for plotting, differential expression testing,
    clustering, and many other downstream analyses. The following review
    provides an overview of RNA-seq data analysis, including information about
    read count matrices:

    > Conesa, A., Madrigal, P., Tarazona, S. et al. A survey of best practices
    > for RNA-seq data analysis. Genome Biol 17, 13 (2016).
    > https://doi.org/10.1186/s13059-016-0881-8"""

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
    """FastQC

    # Quality-check sequencing data with FastQC

    FastQC aims to provide a simple way to do some quality control checks on
    raw sequence data coming from high throughput sequencing pipelines. It
    provides a modular set of analyses which you can use to give a quick
    impression of whether your data has any problems of which you should be
    aware before doing any further analysis.

    *Description taken from [FastQC webpage](https://www.bioinformatics.babraham.ac.uk/projects/fastqc/).*"""

    print("Running fastqc...")

    label = data.static.label
    in_path = data.dynamic.path

    RUN(f"mkdir -p output/{label}/fastqc")
    RUN(f"fastqc -t 8 -o output/{label}/fastqc {in_path}/*.fastq*")

    return data.dynamic


@Function(
    "data.qc = false",
    "ret.label = data.label",
    "ret.qc = true",
)
def multiqc(data: RNASeq, ret: RNASeq.S) -> RNASeq.D:
    """MultiQC

    # Quality-check sequencing data with MultiQC

    MultiQC aggregates the output of [FastQC](https://www.bioinformatics.babraham.ac.uk/projects/fastqc/)
    quality-control checks into a single page viewable in a web browser."""

    print("Running fastqc and multiqc...")

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
    """cutadapt (Illumina RNA-seq)

    # Remove Illumina universal adaptor and poly-A tails with cutadapt

    Cutadapt finds and removes adapter sequences, primers, poly-A tails and
    other types of unwanted sequence from your high-throughput sequencing
    reads.

    Cleaning your data in this way is often required: Reads from small-RNA
    sequencing contain the 3’ sequencing adapter because the read is longer
    than the molecule that is sequenced. Amplicon reads start with a primer
    sequence. Poly-A tails are useful for pulling out RNA from your sample, but
    often you don’t want them to be in your reads.

    *Description taken from [cutadapt documentation](https://cutadapt.readthedocs.io/en/stable/).*

    ## Use of cutadapt for this step

    This particular step of the pipeline will use cutadapt to remove the
    Illumina universal adaptor and poly-A tails. This is suitable for RNA-seq
    experiments with raw data coming from an Illumina machine."""

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
    """kallisto

    # Quantify transcript abundances *without* alignment using kallisto

    kallisto is a program for quantifying abundances of transcripts from
    RNA-Seq data, or more generally of target sequences using high-throughput
    sequencing reads. It is based on the novel idea of pseudoalignment for
    rapidly determining the compatibility of reads with targets, without the
    need for alignment. On benchmarks with standard RNA-Seq data, kallisto can
    quantify 30 million human bulk RNA-seq reads in less than 3 minutes on a
    Mac desktop computer using only the read sequences and a transcriptome
    index that itself takes than 10 minutes to build. Pseudoalignment of reads
    preserves the key information needed for quantification, and kallisto is
    therefore not only fast, but also comparably accurate to other existing
    quantification tools. In fact, because the pseudoalignment procedure is
    robust to errors in the reads, in many benchmarks kallisto significantly
    outperforms existing tools. The kallisto algorithms are described in more
    detail in:

    > NL Bray, H Pimentel, P Melsted and L Pachter, Near optimal probabilistic
    > RNA-seq quantification, Nature Biotechnology 34, p 525--527 (2016).

    *Description taken from [kallisto GitHub repository](https://github.com/pachterlab/kallisto).*"""

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
    """salmon

    # Quantify transcript abundances *without* alignment using salmon

    Salmon is a wicked-fast program to produce a highly-accurate,
    transcript-level quantification estimates from RNA-seq data. Salmon
    achieves its accuracy and speed via a number of different innovations,
    including the use of selective-alignment (accurate but fast-to-compute
    proxies for traditional read alignments), and massively-parallel stochastic
    collapsed variational inference. The result is a versatile tool that fits
    nicely into many different pipelines. For example, you can choose to make
    use of our selective-alignment algorithm by providing Salmon with raw
    sequencing reads, or, if it is more convenient, you can provide Salmon with
    regular alignments (e.g. an unsorted BAM file with alignments to the
    transcriptome produced with your favorite aligner), and it will use the
    same wicked-fast, state-of-the-art inference algorithm to estimate
    transcript-level abundances for your experiment.

    *Description taken from [salmon GitHub repository](https://github.com/COMBINE-lab/salmon).*"""
    pass


@Function(
    "ret.label = data.label",
)
def combat_seq(
    data: TranscriptMatrices, ret: TranscriptMatrices.S
) -> TranscriptMatrices.D:
    """ComBat-seq

    # Correct for batch effects using ComBat-seq

    ComBat-seq is a batch effect adjustment tool for bulk RNA-seq count data.
    It is an improved model based on the popular
        [ComBat](https://doi.org/10.1093/biostatistics/kxj037),
    to address its limitations through novel methods designed specifically for
    RNA-Seq studies.  ComBat-seq takes untransformed, raw count matrix as
    input. Same as ComBat, it requires a known batch variable.

    We use a negative binomial regression to model batch effects, then provide
    adjusted data by mapping the original data to an expected distribution if
    there were no batch effects. This approach better captures the properties
    of RNA-Seq count data compared to the Gaussian distribution assumed by
    ComBat. ComBat-seq specifies different dispersion parameters across
    batches, allowing for flexible modeling of the variance of gene expression.
    In addition, ComBat-seq provides adjusted data which preserves the integer
    nature of counts, so that the adjusted data are compatible with the
    assumptions of state-of-the-art differential expression software (e.g.
    edgeR, DESeq2, which specifically request untransformed count data).

    ComBat-seq was recently published in NAR genomics and bioinformatics.
    Whenever using ComBat-seq, please cite:

    > Yuqing Zhang, Giovanni Parmigiani, W Evan Johnson, ComBat-seq: batch
    > effect adjustment for RNA-seq count data, NAR Genomics and Bioinformatics,
    > Volume 2, Issue 3, 1 September 2020, lqaa078,
    > [https://doi.org/10.1093/nargab/lqaa078](https://doi.org/10.1093/nargab/lqaa078)

    *Description taken from [ComBat-seq GitHub repository](https://github.com/zhangyuqing/ComBat-seq).*"""
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
    """featureCounts

    # Quantify transcript abundances *after* alignment using featureCounts

    featureCounts is a highly efficient general-purpose read summarization
    program that counts mapped reads for genomic features such as genes, exons,
    promoter, gene bodies, genomic bins and chromosomal locations. It can be
    used to count both RNA-seq and genomic DNA-seq reads.

    *Description taken from [featureCounts website](https://subread.sourceforge.net/featureCounts.html).*"""
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
