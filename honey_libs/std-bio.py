import os
import polars as pl

from honey_lang import Helper, Input, Output, Function, __hb_bash


@Helper
class Dir:
    stage = 1

    def make(name):
        dir = f"output/{Dir.stage:03 * 10}-{name}"
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


################################################################################
# %% Raw RNA-seq data (reads)


@Input
class SraRnaSeq:
    "RNA-seq (stored on remote Sequence Read Archive)"

    sample_sheet: str
    "Path to sample sheet CSV with SRA metadata (columns: sample_name, condition, replicate)"


@Input
class LocalRnaSeq:
    "RNA-seq (locally-saved)"

    sample_sheet: str
    "Path to sample sheet CSV (columns: sample_name, condition, replicate)"

    path: str
    "Path to the directory containing the RNA-seq data"


@Output
class RnaSeqReads:
    """@intermediate:RNA-seq reads

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

    path: str

    qc: bool
    trimmed: bool


@Function(
    "ret.qc = false",
    "ret.trimmed = false",
)
def from_sra_rna_seq(__hb_sra: SraRnaSeq, __hb_ret: RnaSeqReads):
    """Download from ENA

    # Download RNA-seq data from the [European Nucleotide Archive](https://www.ebi.ac.uk/ena/browser/home) by SRR accession identifiers

    The downloaded files will be in the .fastq.gz file format, with the
    filenames for the forward reads ending in _1.fastq.gz and the filenames for
    the reverse reads ending in _2.fastq.gz."""

    sample_sheet = pl.read_csv(__hb_sra.sample_sheet)

    for srr in sample_sheet["sample_name"]:
        base_url = "ftp://ftp.sra.ebi.ac.uk/vol1/fastq/"
        base_url += srr[:6] + "/"
        base_url += srr[9:].zfill(3) + "/"
        base_url += srr + "/"

        # Assumes forward (_1) and reverse (_2) reads exist

        __hb_bash(f"""
             wget -nc --directory-prefix={__hb_ret.path} {base_url}{srr}_1.fastq.gz
        """)

        __hb_bash(f"""
             wget -nc --directory-prefix={__hb_ret.path} {base_url}{srr}_2.fastq.gz
        """)

    os.symlink(
        src=__hb_sra.sample_sheet,
        dst=f"{__hb_ret.path}/sample_sheet.csv",
    )


@Function(
    "ret.qc = false",
    "ret.trimmed = false",
)
def load_local_rna_seq(__hb_local: LocalRnaSeq, __hb_ret: RnaSeqReads):
    """Load local data

    # Load raw RNA-seq data already present on your computer

    The raw RNA-seq files are typically in the .fastq.gz file format."""

    carry_over(__hb_local, __hb_ret)

    os.symlink(
        src=__hb_local.sample_sheet,
        dst=f"{__hb_ret.path}/sample_sheet.csv",
    )


@Function(
    "ret.trimmed = reads.trimmed",
    "ret.qc = true",
    "reads.qc = false",
    citation="Simon Andrews. FastQC: a quality control tool for high "
    "throughput sequence data. (2010). Available online at: "
    "http://www.bioinformatics.babraham.ac.uk/projects/fastqc",
    use="a widely-used quality control tool for high throughput sequence data.",
)
def fastqc(__hb_reads: RnaSeqReads, __hb_ret: RnaSeqReads):
    """FastQC

    # Run quality control checks on RNA-seq data with [FastQC](https://www.bioinformatics.babraham.ac.uk/projects/fastqc/)

    FastQC produces two HTML reports for each sample: one for the forward reads
    and one for the reverse reads. These HTML reports can be individually opened
    and inspected in your web browser.

    The Harvard Chan Bioinformatics Core provides a
    [useful tutorial](https://hbctraining.github.io/Intro-to-rnaseq-hpc-salmon/lessons/qc_fastqc_assessment.html#assessing-quality-metrics)
    for assessing the outputs of FastQC."""

    # PARAMETER: The number of cores that you want FastQC to use
    FASTQC_CORES = 4

    carry_over(__hb_reads, __hb_ret)

    sample_sheet = pl.read_csv(f"{__hb_reads.path}/sample_sheet.csv")
    fastqs = " ".join(
        (__hb_reads.path + "/" + sample_sheet["sample_name"] + "_1.fastq.gz ")
        + (__hb_reads.path + "/" + sample_sheet["sample_name"] + "_2.fastq.gz")
    )

    __hb_bash(f"fastqc -t {FASTQC_CORES} -o {__hb_ret.path} {fastqs}")


@Function(
    "ret.trimmed = reads.trimmed",
    "ret.qc = true",
    "reads.qc = false",
    google_scholar_id="15898044054356823756",
    citation="Philip Ewels, Måns Magnusson, Sverker Lundin and Max Käller. "
    "MultiQC: Summarize analysis results for multiple tools and samples in a "
    "single report. Bioinformatics (2016). doi: 10.1093/bioinformatics/btw354. "
    "PMID: 27312411",
    additional_citations=[
        "Simon Andrews. FastQC: a quality control tool for high throughput "
        "sequence data. (2010). Available online at: "
        "http://www.bioinformatics.babraham.ac.uk/projects/fastqc",
    ],
    use="a wrapper around FastQC that also combines all results into a single page.",
)
def multiqc(__hb_reads: RnaSeqReads, __hb_ret: RnaSeqReads):
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
    MultiQC."""

    FASTQC_CORES = 4

    carry_over(__hb_reads, __hb_ret)

    sample_sheet = pl.read_csv(f"{__hb_reads.path}/sample_sheet.csv")
    fastqs = " ".join(
        (__hb_reads.path + "/" + sample_sheet["sample_name"] + "_1.fastq.gz ")
        + (__hb_reads.path + "/" + sample_sheet["sample_name"] + "_2.fastq.gz")
    )

    __hb_bash(f"fastqc -t {FASTQC_CORES} -o {__hb_ret.path} {fastqs}")
    __hb_bash(f"uv run multiqc --filename {__hb_ret.path}/multiqc.html {__hb_ret.path}")


@Function(
    "reads.qc = true",
    "ret.qc = false",
    "reads.trimmed = false",
    "ret.trimmed = true",
    google_scholar_id="4180123542769751602",
    citation="Marcel Martin. Cutadapt removes adapter sequences from "
    "high-throughput sequencing reads. EMBnet.Journal, 17(1):10-12, May 2011. "
    "http://dx.doi.org/10.14806/ej.17.1.200",
)
def cutadapt_illumina(__hb_reads: RnaSeqReads, __hb_ret: RnaSeqReads):
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
    > sequences in an error-tolerant way."""

    carry_over(__hb_reads, __hb_ret, file="sample_sheet.csv")

    sample_sheet = pl.read_csv(f"{__hb_reads.path}/sample_sheet.csv")

    for sample_name in sample_sheet["sample_name"]:
        __hb_bash(f"""uv run cutadapt \\
                    --cores=0 \\
                    -m 1 \\
                    --poly-a \\
                    -a AGATCGGAAGAGCACACGTCTGAACTCCAGTCA \\
                    -A AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT \\
                    -o {__hb_ret.path}/{sample_name}_1.fastq.gz \\
                    -p {__hb_ret.path}/{sample_name}_2.fastq.gz \\
                    {__hb_reads.path}/{sample_name}_1.fastq.gz \\
                    {__hb_reads.path}/{sample_name}_2.fastq.gz""")


################################################################################
# %% Transcript matrices


@Output
class TranscriptMatrices:
    """Transcript read counts (and TPM abundance) of RNA-seq samples

    The goal of this step is to calculate two transcript-by-sample matrices:
    - One with (estimated) read counts.
    - One with TPM (transcripts-per-million) abundance.

    These matrices are usually computed by using a reference transcriptome
    (coding sequences) rather than a reference genome. Unless your scientific
    question relates specifically to transcript information, these matrices are
    often aggregated into **gene-level** read count and abundance information."""

    path: str


@Function(
    "reads.qc = true",
    google_scholar_id="15817796957364212470",
    citation="NL Bray, H Pimentel, P Melsted and L Pachter, Near optimal "
    "probabilistic RNA-seq quantification, Nature Biotechnology 34, "
    "p 525--527 (2016).",
)
def kallisto(__hb_reads: RnaSeqReads, __hb_ret: TranscriptMatrices):
    """kallisto

    # Quantify transcript abundances *without* alignment using [kallisto](https://pachterlab.github.io/kallisto/)

    kallisto is a tool that estimates the number of times a transcript appears
    using a technique called _pseudoalignment_ that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s."""

    # PARAMETER: The location of the kallisto transcriptome index on your computer
    KALLISTO_INDEX = "ensembl115.Homo_sapiens.GRCh38.cdna.all.kallisto.idx"

    # PARAMETER: The number of cores that you want kallisto to use
    KALLISTO_CORES = 4

    carry_over(__hb_reads, __hb_ret, file="sample_sheet.csv")

    sample_sheet = pl.read_csv(f"{__hb_reads.path}/sample_sheet.csv")

    for sample_name in sample_sheet["sample_name"]:
        __hb_bash(f"""kallisto quant \\
                    -t {KALLISTO_CORES} \\
                    -i {KALLISTO_INDEX} \\
                    -o {__hb_ret.path}/{sample_name} \\
                    {__hb_reads.path}/{sample_name}_1.fastq.gz \\
                    {__hb_reads.path}/{sample_name}_2.fastq.gz""")


################################################################################
# %% Gene matrices


@Output
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

    path: str


@Function(
    google_scholar_id="5898741618830664005",
    citation="Soneson C, Love MI, Robinson MD (2015). Differential analyses "
    "for RNA-seq: transcript-level estimates improve gene-level inferences. "
    "F1000Research, 4. doi:10.12688/f1000research.7563.1.",
)
def tximport(__hb_data: TranscriptMatrices, __hb_ret: GeneMatrices):
    """tximport

    # Aggregate transcript-level estimated counts for gene-level analysis with [tximport](https://bioconductor.org/packages/release/bioc/html/tximport.html)

    Tools like [kallisto](https://pachterlab.github.io/kallisto/) and
    [salmon](https://salmon.readthedocs.io/en/latest/) report transcript-level
    read counts, but many analyses of interest (such as differential _gene_
    expression) require gene-level data. tximport aggregates transcripts of the
    same gene together for gene-level downstream analysis.

    Salmon is a tool that estimates the number of times a transcript appears
    using a lightweight mapping technique that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s."""

    # PARAMETER: The version of Ensembl to use for gene annotations
    ENSEMBL_VERSION = "115"

    # PARAMETER: The Ensembl gene annotation dataset to use
    ENSEMBL_DATASET = "hsapiens_gene_ensembl"

    carry_over(__hb_data, __hb_ret, file="sample_sheet.csv")

    __hb_bash(f"""
        Rscript tximport.r \\
            {ENSEMBL_VERSION} \\
            {ENSEMBL_DATASET} \\
            {__hb_data.path}/sample_sheet.csv \\
            {__hb_data.path} \\
            {__hb_ret.path}""")


################################################################################
# %% Differential Gene Expression


@Output
class DifferentialGeneExpression:
    """Differential gene expression

    The goal of this step is to assign a score (like a _p_-value) to each gene
    that ranks how differentially expressed it is between two conditions.
    Among other uses, this information can be plotted in an
    [MA plot](https://en.wikipedia.org/wiki/MA_plot) or a
    [volcano plot](https://en.wikipedia.org/wiki/Volcano_plot_(statistics%29).

    The following image shows a _typical_ RNA-seq processing workflow,
    **but the details can vary a lot!**

    ![An overview of the RNA-seq workflow.](assets/rna-seq.png)

    For example, sometimes you start with reads already quantified or trimmed,
    and sometimes you need to run additional processing steps like batch
    correction."""

    path: str

    comparison_sheet: str
    "@nosuggest:Path to CSV of comparisons to make (columns: control_condition, treatment_condition)"


@Function(
    google_scholar_id="16121678637925818947",
    citation="Love MI, Huber W, Anders S (2014). Moderated estimation of fold change "
    "and dispersion for RNA-seq data with DESeq2. Genome Biology, 15, 550. "
    "doi:10.1186/s13059-014-0550-8.",
    use="a **widely-used tool** that does **not** give you error bars.",
)
def deseq2(__hb_data: GeneMatrices, __hb_ret: DifferentialGeneExpression):
    """DESeq2

    # Find differentially-expressed protein-coding genes with [DESeq2](https://bioconductor.org/packages/release/bioc/html/DESeq2.html)

    DESeq2 models gene expression using what is called a
    <span class="more-info">
        <span>
            A <b>negative binomial distribution</b> is a statistical distribution
            that models count data with unexplained variance, such as the
            number of mRNA transcripts in a cell.  It looks like an off-center
            normal distribution that is only defined for non-negative integers.
        </span>
        negative binomial distribution</span>,
    which is much more suitable for RNA-seq data than something like a _t_-test.
    DESeq2 has an
    [extensive guide](https://bioconductor.org/packages/release/bioc/vignettes/DESeq2/inst/doc/DESeq2.html)
    about how to use DESeq2 to analyze RNA-seq data.

    The above guide includes a very helpful
    [list of frequently-asked questions](https://bioconductor.org/packages/release/bioc/vignettes/DESeq2/inst/doc/DESeq2.html#frequently-asked-questions),
    including an explanation of why some adjusted _p_-values are `NA` and what
    can be done to turn off that behavior."""

    # PARAMETER: The version of Ensembl to use for gene annotations
    ENSEMBL_VERSION = "115"

    # PARAMETER: The Ensembl gene annotation dataset to use
    ENSEMBL_DATASET = "hsapiens_gene_ensembl"

    carry_over(__hb_data, __hb_ret, file="sample_sheet.csv")

    __hb_bash(f"""
        Rscript deseq2.r \\
            {ENSEMBL_VERSION} \\
            {ENSEMBL_DATASET} \\
            {__hb_data.path}/sample_sheet.csv \\
            {__hb_ret.comparison_sheet} \\
            {__hb_data.path}/counts.csv \\
            {__hb_ret.path}""")


################################################################################
# %% Stubs


@Function(
    "reads.qc = true",
    google_scholar_id="11462947284863466602",
    citation="Patro, R., Duggal, G., Love, M. I., Irizarry, R. A., & "
    "Kingsford, C. (2017). Salmon provides fast and bias-aware quantification "
    "of transcript expression. Nature Methods.",
)
def salmon(__hb_reads: RnaSeqReads, __hb_ret: TranscriptMatrices):
    """Salmon

    # Quantify transcript abundances *without* alignment using [Salmon](https://pachterlab.github.io/kallisto/)

    Salmon is a tool that estimates the number of times a transcript appears
    using a lightweight mapping technique that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s."""

    # PARAMETER: The location of the Salmon transcriptome index on your computer
    SALMON_INDEX = "put the path to the Salmon index here"

    # PARAMETER: The number of cores that you want Salmon to use
    SALMON_CORES = 4

    raise NotImplementedError  # Coming soon!


@Function(
    google_scholar_id="1639708055766929241",
    citation="Harold J. Pimentel, Nicolas Bray, Suzette Puente, Páll Melsted "
    "and Lior Pachter, Differential analysis of RNA-Seq incorporating "
    "quantification uncertainty, Nature Methods (2017), advanced access "
    "http://dx.doi.org/10.1038/nmeth.4324.",
    use="a **lesser-used (but still very common)** tool that **does** give you error bars.",
)
def sleuth(__hb_data: TranscriptMatrices, __hb_ret: DifferentialGeneExpression):
    """sleuth

    # Find differentially-expressed protein-coding genes with [sleuth](https://pachterlab.github.io/sleuth/)

    sleuth can find differentially-expressed genes between samples and can
    incorporate measurements of uncertainty using "bootstrap estimates" from
    read quantifiers like [kallisto](http://pachterlab.github.io/kallisto).
    sleuth also has a collection of [walkthroughs](https://pachterlab.github.io/sleuth/walkthroughs)
    that demonstrate how to use it to analyze RNA-seq datasets."""

    raise NotImplementedError  # Coming soon!


# @Function(
#     "ret.label = data.label",
#     "data.bc = false",
#     "ret.bc = true",
# )
# def combat_seq(
#     data: TranscriptMatrices, ret: TranscriptMatrices.S
# ) -> TranscriptMatrices.D:
#     raise NotImplementedError

# @Output
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
#     "ret.bc = false", # TODO why does removing this crash HB?
# )
# def featureCounts(data: Alignment, ret: TranscriptMatrices.S) -> TranscriptMatrices.D:
#     """featureCounts
#
#     # Summarize reads aligned to a reference genome using [featureCounts](https://subread.sourceforge.net/featureCounts.html)
#
#     featureCounts counts up reads mapped to genomic regions. When used with
#     RNA-seq reads that have been aligned with a transcript aligner like
#     [STAR](https://github.com/alexdobin/STAR), these counts correspond to
#     transcript counts. Thus, using featureCounts **requires using an
#     alignment-based approach for RNA-seq read quantification.**
#
#     ## Citation
#
#     If you use featureCounts, please cite it as:
#
#     >  Liao Y, Smyth GK and Shi W (2014). featureCounts: an efficient general
#     > purpose program for assigning sequence reads to genomic features.
#     > Bioinformatics, 30(7):923-30."""
#
#     raise NotImplementedError # Coming soon!
#
#
# @Function(
#     "data.qc = true",
#     "ret.label = data.label",
# )
# def star(data: RnaSeq, ret: Alignment.S) -> Alignment.D:
#     "STAR"
#     pass
#
# @Prop
# class CutAndRunProp:
#     "CUT&RUN-seq"
#
#     label: str
#     "Label for data"
#
#     sample_sheet: str
#     "Path to sample sheet CSV"
#
#     raw_data: str
#     "Path to raw FASTQ files"
#
#
# @Prop
# class EMSeqProp:
#     "EM-seq"
#
#     label: str
#     "Label for data"
#
#     sample_sheet: str
#     "Path to sample sheet CSV"
#
#     raw_data: str
#     "Path to raw FASTQ files"
#
#
# @Prop
# class FlowProp:
#     "Flow cytometry"
#
#     label: str
#     "Label for data"
#
#     sample_sheet: str
#     "Path to sample sheet CSV"
#
#     raw_data: str
#     "Path to raw FCS files"
#
#
# @Prop
# class SortProp:
#     "Sort cells with FACS"
#
#     label: str
#     "Label for data"
#
#
# @Prop
# class StainProp:
#     "Stain cells with antibodies"
#
#     label: str
#     "Label for data"
#
# @Prop
# class TransfectProp:
#     "Infect cells with CRISPR sgRNA guide library"
#
#     label: str
#     "Label for data"
#
#     library: str
#     "Path to the library file"
