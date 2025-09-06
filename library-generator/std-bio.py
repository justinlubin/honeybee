from lib import Function, Helper, Prop, Type


@Helper
def bash(command):
    import subprocess

    print(f"Running bash command:\n\n{command}\n")

    subprocess.run(
        command,
        shell=True,
        text=True,
    )


@Helper
def capture_bash(command):
    import subprocess

    print(f"Running bash command:\n\n{command}\n")

    return subprocess.run(
        command,
        shell=True,
        capture_output=True,
        text=True,
    ).stdout.split()


################################################################################
# %% Raw RNA-seq data (reads)


@Type
class SraRnaSeq:
    class S:
        label: str
        sample_sheet: str

    class D:
        pass


@Prop
class P_SraRnaSeq:
    "RNA-seq (stored on remote Sequence Read Archive)"

    label: str
    "Label for data, like 'main' or 'JL001'"

    sample_sheet: str
    "Path to sample sheet CSV with SRA metadata (columns: srr,condition,replicate)"


@Function(
    "P_SraRnaSeq { label = ret.label, sample_sheet = ret.sample_sheet, raw_data = ret.raw_data }",
)
def F_SraRnaSeq(ret: SraRnaSeq.S) -> SraRnaSeq.D:
    return SraRnaSeq.D()


@Type
class LocalRnaSeq:
    class S:
        label: str
        sample_sheet: str
        path: str

    class D:
        pass


@Prop
class P_LocalRnaSeq:
    "RNA-seq (locally-saved)"

    label: str
    "Label for data, like 'main' or 'JPL001'"

    sample_sheet: str
    "Path to sample sheet CSV (columns: sample_name,condition,replicate)"

    path: str
    "Path to the directory containing the RNA-seq data"


@Function(
    "P_LocalRnaSeq { label = ret.label, sample_sheet = ret.sample_sheet, raw_data = ret.raw_data }",
)
def F_LocalRnaSeq(ret: LocalRnaSeq.S) -> LocalRnaSeq.D:
    return LocalRnaSeq.D()


@Type
class RNASeq:
    """RNA-seq reads

    The goal of this step is to get RNA-seq reads.

    These reads can either be raw data (that is, the direct output of a machine
    like an
    [Illumina sequencer](https://www.illumina.com/systems/sequencing-platforms.html), or
    can be the result of pre-processing that raw data.

    Many pre-processing techniques (like
    [adapter trimming](https://knowledge.illumina.com/software/general/software-general-reference_material-list/000002905))
    require that the RNA-seq reads undergo _quality control_ (QC) checks using
    a tool like
    [FastQC](https://www.bioinformatics.babraham.ac.uk/projects/fastqc/)
    before and after running the tool."""

    class S:
        label: str
        "Label for data"

        qc: bool
        "Whether or not quality checks have been run"

    class D:
        sample_sheet: str
        path: str


@Function(
    "ret.label = local.label",
    "ret.path = local.path",
    "ret.qc = false",
)
def from_local_rna_seq(local: LocalRnaSeq, ret: RNASeq.S) -> RNASeq.D:
    """Load local data

    This function loads raw RNA-seq data that you already have on your computer,
    typically in the .fastq.gz file format."""

    print("### Loading local RNA-seq data files... ###\n")

    return RNASeq.D(
        sample_sheet=local.static.sample_sheet,
        path=local.static.path,
    )


@Function(
    "ret.label = local.label",
    "ret.qc = false",
)
def from_sra_rna_seq(sra: SraRnaSeq, ret: RNASeq.S) -> RNASeq.D:
    """Download from ENA

    This function loads RNA-seq data from the
    [European Nucleotide Archive](https://www.ebi.ac.uk/ena/browser/home) by
    SRR accession identifiers."""

    print("### Downloading RNA-seq data files from ENA... ###\n")

    import polars as pl

    df = pl.read_csv(sra.static.sample_sheet)

    outdir = f"output/{sra.static.label}/sra/"

    for srr in df["srr"]:
        base_url = "ftp://ftp.sra.ebi.ac.uk/vol1/fastq/"
        base_url += srr[:6] + "/"
        base_url += srr[9:].zfill(3) + "/"
        base_url += srr + "/"

        # Assumes forward (_1) and reverse (_2) reads exist

        bash(f"""
             wget -nc --directory-prefix={outdir} {base_url}{srr}_1.fastq.gz
        """)

        bash(f"""
             wget -nc --directory-prefix={outdir} {base_url}{srr}_2.fastq.gz
        """)

    df.with_columns().rename({"srr": "sample_name"}).write_csv(
        outdir + "sample_sheet.csv"
    )

    return RNASeq.D(
        sample_sheet=outdir + "sample_sheet.csv",
        path=outdir,
    )


@Function(
    "data.qc = false",
    "ret.label = data.label",
    "ret.qc = true",
)
def fastqc(data: RNASeq, ret: RNASeq.S) -> RNASeq.D:
    """FastQC

    Run quality control checks on RNA-seq data with
    [FastQC](https://www.bioinformatics.babraham.ac.uk/projects/fastqc/).

    FastQC produces two HTML reports for each sample: one for the forward reads
    and one for the reverse reads. These HTML reports can be individually opened
    and inspected in your web browser.

    The Harvard Chan Bioinformatics Core provides a
    [useful tutorial](https://hbctraining.github.io/Intro-to-rnaseq-hpc-salmon/lessons/qc_fastqc_assessment.html#assessing-quality-metrics)
    for assessing the outputs of FastQC.

    ## Citation

    If you use FastQC, please cite it as:

    > Simon Andrews. FastQC: a quality control tool for high throughput
    > sequence data. (2010). Available online at:
    > http://www.bioinformatics.babraham.ac.uk/projects/fastqc"""

    print("### Running FastQC... ###\n")

    outdir = f"output/{ret.label}/fastqc/"
    bash(f"mkdir -p {outdir}")

    import polars as pl

    df = pl.read_csv(data.dynamic.sample_sheet)
    fastqs = " ".join(
        (data.dynamic.path + df["sample_name"] + "_1.fastq.gz ")
        + (data.dynamic.path + df["sample_name"] + "_2.fastq.gz")
    )
    cores = 8

    bash(f"fastqc -t {cores} -o {outdir} {fastqs}")

    return data.dynamic


@Function(
    "data.qc = false",
    "ret.label = data.label",
    "ret.qc = true",
)
def multiqc(data: RNASeq, ret: RNASeq.S) -> RNASeq.D:
    """MultiQC

    Run quality control checks on RNA-seq data with
    [MultiQC](https://seqera.io/multiqc/).

    MultiQC aggregates the output of
    [FastQC](https://www.bioinformatics.babraham.ac.uk/projects/fastqc/)
    quality-control checks into a single page viewable in a web browser.

    The Harvard Chan Bioinformatics Core provides a
    [useful tutorial](https://hbctraining.github.io/Intro-to-rnaseq-hpc-salmon/lessons/qc_fastqc_assessment.html#assessing-quality-metrics)
    for assessing the outputs of FastQC that can also be used to understand
    the outputs of MultiQC.

    ## Citation

    If you use MultiQC, please cite it as:

    > Philip Ewels, Måns Magnusson, Sverker Lundin and Max Käller. MultiQC:
    > Summarize analysis results for multiple tools and samples in a single
    > report. Bioinformatics (2016). doi: 10.1093/bioinformatics/btw354.
    > PMID: 27312411

    This step also relies on FastQC. Please also cite it as:

    > Simon Andrews. FastQC: a quality control tool for high throughput
    > sequence data. (2010). Available online at:
    > http://www.bioinformatics.babraham.ac.uk/projects/fastqc"""

    print("### Running MultiQC (and pre-requisite FastQC commands)... ###")

    fastqc_outdir = f"output/{ret.label}/fastqc/"
    bash(f"mkdir -p {fastqc_outdir}")

    import polars as pl

    df = pl.read_csv(data.dynamic.sample_sheet)
    fastqs = " ".join(
        (data.dynamic.path + df["sample_name"] + "_1.fastq.gz ")
        + (data.dynamic.path + df["sample_name"] + "_2.fastq.gz")
    )
    cores = 8

    bash(f"fastqc -t {cores} -o {fastqc_outdir} {fastqs}")

    multiqc_outdir = f"output/{ret.label}/multiqc/"
    bash(f"mkdir -p {multiqc_outdir}")
    bash(f"uv run multiqc --filename {multiqc_outdir}multiqc.html {fastqc_outdir}")

    return data.dynamic


@Function(
    "data.qc = true",
    "ret.label = data.label",
    "ret.qc = false",
)
def cutadapt_illumina(data: RNASeq, ret: RNASeq.S) -> RNASeq.D:
    """cutadapt (Illumina)

    Remove the Illumina universal adapter for RNA-seq and poly(A) tails from
    an RNA-seq dataset using [cutadapt](https://cutadapt.readthedocs.io/en/stable/).

    This is typically a good step to do in an RNA-seq pre-processing pipeline.
    [Adapter trimming](https://knowledge.illumina.com/library-preparation/general/library-preparation-general-reference_material-list/000001314)
    removes adapter sequences that are present due to a read length being
    longer than the insert size of the sequence in a sequencer. Poly(A) tails
    are the result of post-transcriptional
    [polyadenylation](https://www.nature.com/articles/nsb1000_838),
    and thus will not map back to a reference genome or transcriptome;
    therefore, if you're not specifically looking to analyze polyadenylation,
    you'll likely want to remove these tails for your analysis.

    From the [cutadapt manual](https://cutadapt.readthedocs.io/en/stable/):

    > Cutadapt finds and removes adapter sequences, primers, poly-A tails and
    > other types of unwanted sequence from your high-throughput sequencing
    > reads.

    > Cleaning your data in this way is often required: Reads from small-RNA
    > sequencing contain the 3’ sequencing adapter because the read is longer
    > than the molecule that is sequenced. Amplicon reads start with a primer
    > sequence. Poly-A tails are useful for pulling out RNA from your sample,
    > but often you don’t want them to be in your reads.

    > Cutadapt helps with these trimming tasks by finding the adapter or primer
    > sequences in an error-tolerant way. It can also modify and filter
    > single-end and paired-end reads in various ways. Adapter sequences can
    > contain IUPAC wildcard characters. Cutadapt can also demultiplex your
    > reads.

    ## Citation

    If you use cutadapt, please cite it as:

    > Marcel Martin. Cutadapt removes adapter sequences from high-throughput
    > sequencing reads. EMBnet.Journal, 17(1):10-12, May 2011.
    > http://dx.doi.org/10.14806/ej.17.1.200"""

    print("### Running cudapat (Illumina RNA-seq)... ###")

    outdir = f"output/{ret.label}/cutadapt_trimmed/"
    bash(f"mkdir -p {outdir}")

    import polars as pl

    df = pl.read_csv(data.dynamic.sample_sheet)

    for sample_name in df["sample_name"]:
        bash(f"""uv run cutadapt \\
                    --cores=0 \\
                    -m 1 \\
                    --poly-a \\
                    -a AGATCGGAAGAGCACACGTCTGAACTCCAGTCA \\
                    -A AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT \\
                    -o {outdir}{sample_name}_1.fastq.gz \\
                    -p {outdir}{sample_name}_2.fastq.gz \\
                    {data.dynamic.path}{sample_name}_1.fastq.gz \\
                    {data.dynamic.path}{sample_name}_2.fastq.gz""")

    return RNASeq.D(
        sample_sheet=data.dynamic.sample_sheet,
        path=outdir,
    )


################################################################################
# %% Transcript matrices


@Type
class TranscriptMatrices:
    """Read count (and TPM abundance) matrix for RNA-seq samples

    The goal of this step is to calculate two transcript × sample matrices:
    - One with (estimated) read counts.
    - One with TPM (transcripts-per-million) abundance.

    These matrices can be used for plotting, differential expression testing,
    clustering, and many other downstream analyses. The following review
    provides an overview of RNA-seq data analysis, including information about
    read count matrices:

    > Conesa, A., Madrigal, P., Tarazona, S. et al. A survey of best practices
    > for RNA-seq data analysis. Genome Biol 17, 13 (2016).
    > https://doi.org/10.1186/s13059-016-0881-8"""

    class S:
        label: str
        "Label for RNA-seq data to analyze"

    class D:
        sample_sheet: str
        path: str


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

    class S:
        label: str
        "Label for data"

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
