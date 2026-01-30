# %% Helpers and types

from dataclasses import dataclass


def bash(command):
    import subprocess

    print(f"Running bash command:\n\n{command}\n")

    subprocess.run(
        command,
        shell=True,
        text=True,
    )


def capture_bash(command):
    import subprocess

    print(f"Running bash command:\n\n{command}\n")

    return subprocess.run(
        command,
        shell=True,
        capture_output=True,
        text=True,
    ).stdout.split()


@dataclass
class SraRnaSeq:
    @dataclass
    class S:
        label: str
        sample_sheet: str

    @dataclass
    class D:
        pass

    static: S
    dynamic: D


@dataclass
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

    static: S
    dynamic: D


@dataclass
class TranscriptMatrices:
    """Transcript read counts (and TPM abundance) of RNA-seq samples

    The goal of this step is to calculate two transcript-by-sample matrices:
    - One with (estimated) read counts.
    - One with TPM (transcripts-per-million) abundance.

    These matrices are usually computed by using a reference transcriptome
    (coding sequences) rather than a reference genome. Unless your scientific
    question relates specifically to transcript information, these matrices are
    often aggregated into **gene-level** read count and abundance information."""

    @dataclass
    class S:
        label: str
        "Label for RNA-seq data to analyze"

        bc: bool
        "Whether or not batch correction has been run"

    @dataclass
    class D:
        sample_sheet: str
        path: str

    static: S
    dynamic: D


@dataclass
class GeneMatrices:
    """Gene read counts (and TPM abundance) of RNA-seq samples

    The goal of this step is to calculate two gene-by-sample matrices:
    - One with (estimated) read counts.
    - One with TPM (transcripts-per-million) abundance.

    These matrices can be created using an alignment-based approach that aligns
    transcripts to a reference genome in a splice-aware fashion or using
    an alignment-free approach that matches transcripts against a reference
    transcriptome.

    If you have a reference transcriptome already available that you trust and
    you are not specifically interested in scientifically studying the
    alignment of your RNA-seq to the genome, then a tool that performs
    quantification without alignment is generally a good choice due to their
    orders-of-magnitude speedup over alignment-based procedures.

    These matrices can be used for plotting, differential expression testing,
    clustering, and many other downstream analyses. The following review
    provides an overview of RNA-seq data analysis, including information about
    read count matrices (Fig 2a and 2b are especially relevant):

    > Conesa, A., Madrigal, P., Tarazona, S. et al. A survey of best practices
    > for RNA-seq data analysis. Genome Biol 17, 13 (2016).
    > https://doi.org/10.1186/s13059-016-0881-8"""

    @dataclass
    class S:
        label: str
        "Label for RNA-seq data to analyze"

        bc: bool
        "Whether or not batch correction has been run"

    @dataclass
    class D:
        sample_sheet: str
        path: str

    static: S
    dynamic: D


@dataclass
class DifferentialGeneExpression:
    """Differential gene expression

    The goal of this step is to assign a score (like a _p_-value) to each gene
    that ranks how differentially expressed it is between two conditions.
    Among other uses, this information can be plotted in an
    [MA plot](https://en.wikipedia.org/wiki/MA_plot) or a
    [volcano plot](https://en.wikipedia.org/wiki/Volcano_plot_(statistics))."""

    @dataclass
    class S:
        label: str
        "Label for data"

        comparison_sheet: str
        "@noauto:Comparisons to make (columns: control_condition, treatment_condition)"

    @dataclass
    class D:
        sample_sheet: str
        path: str

    static: S
    dynamic: D


# %% F_SraRnaSeq


def F_SraRnaSeq(ret: SraRnaSeq.S) -> SraRnaSeq.D:
    return SraRnaSeq.D()


SRARNASEQ = SraRnaSeq(
    static=SraRnaSeq.S(label="main", sample_sheet="samples.csv"),
    dynamic=F_SraRnaSeq(ret=SraRnaSeq.S(label="main", sample_sheet="samples.csv")),
)

SRARNASEQ

# %% Download from ENA


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

        # bash(f"""
        #      wget -nc --directory-prefix={outdir} {base_url}{srr}_1.fastq.gz
        # """)

        # bash(f"""
        #      wget -nc --directory-prefix={outdir} {base_url}{srr}_2.fastq.gz
        # """)

    df.with_columns().rename({"srr": "sample_name"}).write_csv(
        outdir + "sample_sheet.csv"
    )

    return RnaSeq.D(
        sample_sheet=outdir + "sample_sheet.csv",
        path=outdir,
    )


RNASEQ4 = RnaSeq(
    static=RnaSeq.S(label="main", qc=False),
    dynamic=from_sra_rna_seq(sra=SRARNASEQ, ret=RnaSeq.S(label="main", qc=False)),
)

RNASEQ4

# %% FastQC


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

    - `CORES`: the number of cores that you want FastQC to use (default: 4)

    ## Citation

    If you use FastQC, please cite it as:

    > Simon Andrews. FastQC: a quality control tool for high throughput
    > sequence data. (2010). Available online at:
    > http://www.bioinformatics.babraham.ac.uk/projects/fastqc"""

    CORES = 4

    print("### Running FastQC... ###\n")

    outdir = f"output/{ret.label}/fastqc/"
    bash(f"mkdir -p {outdir}")

    import polars as pl

    df = pl.read_csv(data.dynamic.sample_sheet)
    fastqs = " ".join(
        (data.dynamic.path + df["sample_name"] + "_1.fastq.gz ")
        + (data.dynamic.path + df["sample_name"] + "_2.fastq.gz")
    )

    # bash(f"fastqc -t {CORES} -o {outdir} {fastqs}")

    return data.dynamic


RNASEQ3 = RnaSeq(
    static=RnaSeq.S(label="main", qc=True),
    dynamic=fastqc(data=RNASEQ4, ret=RnaSeq.S(label="main", qc=True)),
)

RNASEQ3

# %% cutadapt (Illumina)


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
    > sequences in an error-tolerant way.

    ## Citation

    If you use cutadapt, please cite it as:

    > Marcel Martin. Cutadapt removes adapter sequences from high-throughput
    > sequencing reads. EMBnet.Journal, 17(1):10-12, May 2011.
    > http://dx.doi.org/10.14806/ej.17.1.200"""

    print("### Running cutadapt (Illumina RNA-seq)... ###")

    outdir = f"output/{ret.label}/cutadapt/"
    bash(f"mkdir -p {outdir}")

    import polars as pl

    df = pl.read_csv(data.dynamic.sample_sheet)

    for sample_name in df["sample_name"]:
        continue
        # bash(f"""uv run cutadapt \\
        #             --cores=0 \\
        #             -m 1 \\
        #             --poly-a \\
        #             -a AGATCGGAAGAGCACACGTCTGAACTCCAGTCA \\
        #             -A AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT \\
        #             -o {outdir}{sample_name}_1.fastq.gz \\
        #             -p {outdir}{sample_name}_2.fastq.gz \\
        #             {data.dynamic.path}{sample_name}_1.fastq.gz \\
        #             {data.dynamic.path}{sample_name}_2.fastq.gz""")

    return RnaSeq.D(
        sample_sheet=data.dynamic.sample_sheet,
        path=outdir,
    )


RNASEQ2 = RnaSeq(
    static=RnaSeq.S(label="main", qc=False),
    dynamic=cutadapt_illumina(data=RNASEQ3, ret=RnaSeq.S(label="main", qc=False)),
)

RNASEQ2

# %% FastQC


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

    - `CORES`: the number of cores that you want FastQC to use (default: 4)

    ## Citation

    If you use FastQC, please cite it as:

    > Simon Andrews. FastQC: a quality control tool for high throughput
    > sequence data. (2010). Available online at:
    > http://www.bioinformatics.babraham.ac.uk/projects/fastqc"""

    CORES = 4

    print("### Running FastQC... ###\n")

    outdir = f"output/{ret.label}/fastqc/"
    bash(f"mkdir -p {outdir}")

    import polars as pl

    df = pl.read_csv(data.dynamic.sample_sheet)
    fastqs = " ".join(
        (data.dynamic.path + df["sample_name"] + "_1.fastq.gz ")
        + (data.dynamic.path + df["sample_name"] + "_2.fastq.gz")
    )

    # bash(f"fastqc -t {CORES} -o {outdir} {fastqs}")

    return data.dynamic


RNASEQ = RnaSeq(
    static=RnaSeq.S(label="main", qc=True),
    dynamic=fastqc(data=RNASEQ2, ret=RnaSeq.S(label="main", qc=True)),
)

RNASEQ

# %% kallisto


def kallisto(data: RnaSeq, ret: TranscriptMatrices.S) -> TranscriptMatrices.D:
    """kallisto

    # Quantify transcript abundances *without* alignment using [kallisto](https://pachterlab.github.io/kallisto/)

    kallisto is a tool that estimates the number of times a transcript appears
    using a technique called _pseudoalignment_ that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s.

    ## Parameters to set

    In the code, please set the following parameters:

    - `KALLISTO_INDEX`: the location of the kallisto transcriptome index on your computer
    - `CORES`: the number of cores that you want kallisto to use (default: 4)

    ## Citation

    If you use kallisto, please cite it as:

    > NL Bray, H Pimentel, P Melsted and L Pachter, Near optimal probabilistic
    > RNA-seq quantification, Nature Biotechnology 34, p 525--527 (2016)."""

    KALLISTO_INDEX = "ensembl115.Homo_sapiens.GRCh38.cdna.all.kallisto.idx"
    CORES = 4

    print("### Running kallisto ###")

    outdir = f"output/{ret.label}/kallisto"
    bash(f"mkdir -p {outdir}")

    import polars as pl

    df = pl.read_csv(data.dynamic.sample_sheet)

    for sample_name in df["sample_name"]:
        continue
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


TRANSCRIPTMATRICES = TranscriptMatrices(
    static=TranscriptMatrices.S(label="main", bc=False),
    dynamic=kallisto(data=RNASEQ, ret=TranscriptMatrices.S(label="main", bc=False)),
)

TRANSCRIPTMATRICES

# %% tximport


def tximport(data: TranscriptMatrices, ret: GeneMatrices.S) -> GeneMatrices.D:
    """tximport

    # Aggregate transcript-level estimated counts for gene-level analysis with [tximport](https://bioconductor.org/packages/release/bioc/html/tximport.html)

    Tools like [kallisto](https://pachterlab.github.io/kallisto/) and
    [salmon](https://salmon.readthedocs.io/en/latest/) report transcript-level
    read counts, but many analyses of interest (such as differential _gene_
    expression) require gene-level data. tximport aggregates transcripts of the
    same gene together for gene-level downstream analysis.

    Salmon is a tool that estimates the number of times a transcript appears
    using a lightweight mapping technique that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s.

    ## Parameters to set

    In the code, please set the following parameters:

    - `ENSEMBL_VERSION`: the version of Ensembl to use for gene annotations (default: "115", released September 2025)
    - `ENSEMBL_DATASET`: the Ensembl gene annotation dataset to use (default: "hsapiens_gene_ensembl")

    ## Citation

    If you use tximport, please cite it as:

    > Soneson C, Love MI, Robinson MD (2015). “Differential analyses for
    RNA-seq: transcript-level estimates improve gene-level inferences.”
    F1000Research, 4. doi:10.12688/f1000research.7563.1."""

    ENSEMBL_VERSION = "115"
    ENSEMBL_DATASET = "hsapiens_gene_ensembl"

    print("### Running tximport... ###\n")

    outdir = f"output/{ret.label}/tximport/"

    # bash(f"""
    #    Rscript tximport.r \\
    #        {ENSEMBL_VERSION} \\
    #        {ENSEMBL_DATASET} \\
    #        {data.dynamic.sample_sheet} \\
    #        {data.dynamic.path} \\
    #        {outdir}""")

    return GeneMatrices.D(
        sample_sheet=data.dynamic.sample_sheet,
        path=outdir,
    )


GENEMATRICES = GeneMatrices(
    static=GeneMatrices.S(label="main", bc=True),
    dynamic=tximport(
        data=TRANSCRIPTMATRICES, ret=GeneMatrices.S(label="main", bc=True)
    ),
)

GENEMATRICES

# %% DESeq2


def deseq2(
    data: GeneMatrices, ret: DifferentialGeneExpression.S
) -> DifferentialGeneExpression.D:
    """DESeq2

    # Find differentially-expressed protein-coding genes with [DESeq2](https://bioconductor.org/packages/release/bioc/html/DESeq2.html)

    DESeq2 models gene expression using what is called a _negative binomial distribution_,
    which is much more suitable for RNA-seq data than something like a _t_-test.
    DESeq2 has an
    [extensive guide](https://bioconductor.org/packages/release/bioc/vignettes/DESeq2/inst/doc/DESeq2.html)
    about how to use DESeq2 to analyze RNA-seq data.

    The above guide includes a very helpful
    [list of frequently-asked questions](https://bioconductor.org/packages/release/bioc/vignettes/DESeq2/inst/doc/DESeq2.html#frequently-asked-questions),
    including an explanation of why some adjusted _p_-values are `NA` and what
    can be done to turn off that behavior.

    ## Parameters to set

    In the code, please set the following parameters:

    - `ENSEMBL_VERSION`: the version of Ensembl to use for gene annotations (default: "115", released September 2025)
    - `ENSEMBL_DATASET`: the Ensembl gene annotation dataset to use (default: "hsapiens_gene_ensembl")

    ## Citation

    If you use DESeq, please cite it as:

    > Love MI, Huber W, Anders S (2014). “Moderated estimation of fold change
    > and dispersion for RNA-seq data with DESeq2.” Genome Biology, 15, 550.
    > doi:10.1186/s13059-014-0550-8."""

    ENSEMBL_VERSION = "115"
    ENSEMBL_DATASET = "hsapiens_gene_ensembl"

    print("### Running DESeq2... ###\n")

    outdir = f"output/{ret.label}/deseq2/"

    bash(f"""
        Rscript deseq2.r \\
            {ENSEMBL_VERSION} \\
            {ENSEMBL_DATASET} \\
            {data.dynamic.sample_sheet} \\
            {ret.comparison_sheet} \\
            {data.dynamic.path}counts.csv \\
            {outdir}""")

    return DifferentialGeneExpression.D(
        sample_sheet=data.dynamic.sample_sheet,
        path=outdir,
    )


GOAL = DifferentialGeneExpression(
    static=DifferentialGeneExpression.S(
        label="main", comparison_sheet="comparisons.csv"
    ),
    dynamic=deseq2(
        data=GENEMATRICES,
        ret=DifferentialGeneExpression.S(
            label="main", comparison_sheet="comparisons.csv"
        ),
    ),
)

GOAL
