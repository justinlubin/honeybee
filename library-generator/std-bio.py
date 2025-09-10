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
    "P_SraRnaSeq { label = ret.label, sample_sheet = ret.sample_sheet }",
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
    "P_LocalRnaSeq { label = ret.label, sample_sheet = ret.sample_sheet, path = ret.path }",
)
def F_LocalRnaSeq(ret: LocalRnaSeq.S) -> LocalRnaSeq.D:
    return LocalRnaSeq.D()


@Type
class RnaSeq:
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
    "ret.qc = false",
)
def from_local_rna_seq(local: LocalRnaSeq, ret: RnaSeq.S) -> RnaSeq.D:
    """Load local data

    # Load raw RNA-seq data already present on your computer

    The raw RNA-seq files are typically in the .fastq.gz file format."""

    print("### Loading local RNA-seq data files... ###\n")

    return RnaSeq.D(
        sample_sheet=local.static.sample_sheet,
        path=local.static.path,
    )


@Function(
    "ret.label = sra.label",
    "ret.qc = false",
)
def from_sra_rna_seq(sra: SraRnaSeq, ret: RnaSeq.S) -> RnaSeq.D:
    """Download from ENA

    # Download RNA-seq data from the [European Nucleotide Archive](https://www.ebi.ac.uk/ena/browser/home) by SRR accession identifiers

    The downloaded files will be in the .fastq.gz file format, with the
    filenames for the forward reads ending in _1.fastq.gz and the filenames for
    the reverse reads ending in _2.fastq.gz."""

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

    return RnaSeq.D(
        sample_sheet=outdir + "sample_sheet.csv",
        path=outdir,
    )


@Function(
    "data.qc = false",
    "ret.label = data.label",
    "ret.qc = true",
)
def fastqc(data: RnaSeq, ret: RnaSeq.S) -> RnaSeq.D:
    """FastQC

    # Run quality control checks on RNA-seq data with [FastQC](https://www.bioinformatics.babraham.ac.uk/projects/fastqc/)

    FastQC produces two HTML reports for each sample: one for the forward reads
    and one for the reverse reads. These HTML reports can be individually opened
    and inspected in your web browser.

    The Harvard Chan Bioinformatics Core provides a
    [useful tutorial](https://hbctraining.github.io/Intro-to-rnaseq-hpc-salmon/lessons/qc_fastqc_assessment.html#assessing-quality-metrics)
    for assessing the outputs of FastQC.

    ## Parameters to set

    In the code, please set the following parameters:

    - `CORES`: the number of cores that you want FastQC to use

    ## Citation

    If you use FastQC, please cite it as:

    > Simon Andrews. FastQC: a quality control tool for high throughput
    > sequence data. (2010). Available online at:
    > http://www.bioinformatics.babraham.ac.uk/projects/fastqc"""

    CORES = 8

    print("### Running FastQC... ###\n")

    outdir = f"output/{ret.label}/fastqc/"
    bash(f"mkdir -p {outdir}")

    import polars as pl

    df = pl.read_csv(data.dynamic.sample_sheet)
    fastqs = " ".join(
        (data.dynamic.path + df["sample_name"] + "_1.fastq.gz ")
        + (data.dynamic.path + df["sample_name"] + "_2.fastq.gz")
    )

    bash(f"fastqc -t {CORES} -o {outdir} {fastqs}")

    return data.dynamic


@Function(
    "data.qc = false",
    "ret.label = data.label",
    "ret.qc = true",
)
def multiqc(data: RnaSeq, ret: RnaSeq.S) -> RnaSeq.D:
    """MultiQC

    # Run quality control checks on RNA-seq data with [MultiQC](https://seqera.io/multiqc/)

    MultiQC aggregates the output of
    [FastQC](https://www.bioinformatics.babraham.ac.uk/projects/fastqc/)
    quality-control checks into a single page viewable in a web browser.

    The Harvard Chan Bioinformatics Core provides a
    [useful tutorial](https://hbctraining.github.io/Intro-to-rnaseq-hpc-salmon/lessons/qc_fastqc_assessment.html#assessing-quality-metrics)
    for assessing the outputs of FastQC that can also be used to understand
    the outputs of MultiQC.

    This function calls FastQC on the necessary files then aggregates them with
    MultiQC.

    ## Parameters to set

    In the code, please set the following parameters:

    - `CORES`: the number of cores that you want FastQC to use

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

    CORES = 8

    print("### Running MultiQC (and pre-requisite FastQC commands)... ###")

    fastqc_outdir = f"output/{ret.label}/fastqc/"
    bash(f"mkdir -p {fastqc_outdir}")

    import polars as pl

    df = pl.read_csv(data.dynamic.sample_sheet)
    fastqs = " ".join(
        (data.dynamic.path + df["sample_name"] + "_1.fastq.gz ")
        + (data.dynamic.path + df["sample_name"] + "_2.fastq.gz")
    )

    bash(f"fastqc -t {CORES} -o {fastqc_outdir} {fastqs}")

    multiqc_outdir = f"output/{ret.label}/multiqc/"
    bash(f"mkdir -p {multiqc_outdir}")
    bash(f"uv run multiqc --filename {multiqc_outdir}multiqc.html {fastqc_outdir}")

    return data.dynamic


@Function(
    "data.qc = true",
    "ret.label = data.label",
    "ret.qc = false",
)
def cutadapt_illumina(data: RnaSeq, ret: RnaSeq.S) -> RnaSeq.D:
    """cutadapt (Illumina)

    # Remove Illumina universal adapter for RNA-seq and poly(A) tails using [cutadapt](https://cutadapt.readthedocs.io/en/stable/).

    This is typically a good step to do in an RNA-seq pre-processing pipeline,
    and **typically only need to be done (at most) once**.
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

    print("### Running cutadapt (Illumina RNA-seq)... ###")

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

    return RnaSeq.D(
        sample_sheet=data.dynamic.sample_sheet,
        path=outdir,
    )


################################################################################
# %% Transcript matrices


@Type
class TranscriptMatrices:
    """Transcript read counts (and TPM abundance) of RNA-seq samples

    The goal of this step is to calculate two transcript-by-sample matrices:
    - One with (estimated) read counts.
    - One with TPM (transcripts-per-million) abundance.

    These matrices can be used for plotting, differential expression testing,
    clustering, and many other downstream analyses. The following review
    provides an overview of RNA-seq data analysis, including information about
    read count matrices (Fig 2a and 2b are especially relevant):

    > Conesa, A., Madrigal, P., Tarazona, S. et al. A survey of best practices
    > for RNA-seq data analysis. Genome Biol 17, 13 (2016).
    > https://doi.org/10.1186/s13059-016-0881-8"""

    class S:
        label: str
        "Label for RNA-seq data to analyze"

        bc: str
        "Whether or not batch correction has been run"

    class D:
        sample_sheet: str
        path: str


@Function(
    "data.qc = true",
    "ret.label = data.label",
    "ret.bc = false",
)
def kallisto(data: RnaSeq, ret: TranscriptMatrices.S) -> TranscriptMatrices.D:
    """kallisto

    # Quantify transcript abundances *without* alignment using [kallisto](https://pachterlab.github.io/kallisto/)

    kallisto is a tool that estimates the number of times a transcript appears
    using a technique called _pseudoalignment_ that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s.

    If you have a reference transcriptome already available that you trust and
    you are not specifically interested in scientifically studying the
    alignment of your RNA-seq to the genome, then a tool that performs
    quantification without alignment (like kallisto or
    [salmon](https://salmon.readthedocs.io/en/latest/)) is generally a good
    choice due to their orders-of-magnitude speedup over alignment-based
    procedures.

    ## Parameters to set

    In the code, please set the following parameters:

    - `KALLISTO_INDEX`: the location of the kallisto index on your computer
    - `CORES`: the number of cores that you want kallisto to use

    ## Citation

    If you use kallisto, please cite it as:

    > NL Bray, H Pimentel, P Melsted and L Pachter, Near optimal probabilistic
    > RNA-seq quantification, Nature Biotechnology 34, p 525--527 (2016)."""

    KALLISTO_INDEX = "put the path to the kallisto index here"
    CORES = 8

    print("### Running kallisto ###")

    outdir = f"output/{ret.label}/kallisto_quant"
    bash(f"mkdir -p {outdir}")

    import polars as pl

    df = pl.read_csv(data.dynamic.sample_sheet)

    for sample_name in df["sample_name"]:
        bash(f"""kallisto quant \\
                    -t {CORES} \\
                    -i {KALLISTO_INDEX} \\
                    -o {outdir}/{sample_name} \\
                    {data.dynamic.path}/{sample_name}_1.fastq.gz \\
                    {data.dynamic.path}/{sample_name}_2.fastq.gz""")

    return TranscriptMatrices.D(
        sample_sheet=data.dynamic.sample_sheet,
        path=outdir,
    )


################################################################################
# %% TODO


@Function(
    "ret.label = data.label",
    "data.bc = false",
    "ret.bc = true",
)
def combat_seq(
    data: TranscriptMatrices, ret: TranscriptMatrices.S
) -> TranscriptMatrices.D:
    pass


@Type
class DifferentialGeneExpression:
    """Differential gene expression

    The goal of this step is to assign a score (like a _p_-value) to each gene
    that ranks how differentially expressed it is between two conditions.
    Among other uses, this information can be plotted in an
    [MA plot](https://en.wikipedia.org/wiki/MA_plot) or a
    [volcano plot](https://en.wikipedia.org/wiki/Volcano_plot_(statistics))."""

    class S:
        label: str
        "Label for data"

    class D:
        sample_sheet: str
        path: str


# @Type
# class Alignment:
#     "Alignment to a reference genome"
#
#     class S:
#         label: str
#         "Label for data"
#
#     class D:
#         sample_sheet: str
#         path: str
#
#
# @Function(
#     "ret.label = data.label",
# )
# def featureCounts(data: Alignment, ret: TranscriptMatrices.S) -> TranscriptMatrices.D:
#     pass
#
#
# @Function(
#     "data.qc = true",
#     "ret.label = data.label",
# )
# def star(data: RnaSeq, ret: Alignment.S) -> Alignment.D:
#     pass
