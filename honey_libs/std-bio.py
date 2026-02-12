import os
import glob
import datetime
import polars as pl

from honey_lang import Helper, Input, Output, Function, __hb_bash

################################################################################
# %% Helper


@Helper
class Dir:
    stage = 1

    @staticmethod
    def make(name):
        dir = f"output/{Dir.stage * 10:03d}-{name}"
        os.makedirs(dir, exist_ok=True)
        Dir.stage += 1
        return dir


@Helper
def carry_over(src_object, dst_object, *, file=None):
    def carry_one(f):
        src = f"../../{src_object.path}/{f}"  # relative to dst
        dst = f"{dst_object.path}/{f}"
        if os.path.islink(src):
            src = os.readlink(src)
        os.symlink(src=src, dst=dst)

    if file is None:
        for file in os.listdir(src_object.path):
            carry_one(file)
    else:
        carry_one(file)


@Helper
def save(src, dst):
    dir = os.path.dirname(dst)
    os.makedirs(dir, exist_ok=True)
    os.symlink(src=os.path.abspath(src), dst=dst)


################################################################################
# %% Sequencing reads


@Output
class SeqReads:
    """@intermediate:Sequencing reads

    The goal of this step is to process sequencing "reads."

    A "read" is either a short (a few hundred base pairs) or long (a few
    thousand base pairs) snippet of DNA produced by a sequencer. This DNA can
    be genomic DNA (perhaps that has undergone some processing, such as in
    [ATAC-seq](https://www.nature.com/articles/nmeth.2688)), or it can be
    derived from RNA in a cell (as in RNA-sequencing).

    Reads are stored typically stored in the `.fastq` (uncompressed) or
    `.fastq.gz` (compressed) file format. Before being able to get a table of
    information (e.g. about chromatin accessibility, methylation status, or
    transcription levels), reads first need to be _preprocessed_. Most
    preprocessing is method-specific, but some methods share commonalities, such
    as _quality control_ (QC) checks."""

    path: str

    qc: bool
    trimmed: bool
    long: bool
    type: str


@Output
class SeqAlignment:
    """Sequence alignment (SAM)

    [SAM files](https://doi.org/10.1093/bioinformatics/btp352) are
    **uncompressed** files that describe an alignment of reads to a reference
    genome. These alignments may or may not be "sorted" by nucleotide position
    and may or may not be "indexed"; an "index" is a supplementary file to the
    SAM file that allows for the computer to quickly access various information
    from the alignment for viewing in, for example, the
    [Integrative Genomics Viewer (IGV)](https://igv.org/)."""

    path: str

    type: str


@Output
class SortedIndexBAM:
    """Sequence alignment (sorted and indexed BAM files)

    [BAM files](https://doi.org/10.1093/bioinformatics/btp352) are
    **compressed** files that describe an alignment of reads to a reference
    genome. These alignments may or may not be "sorted" by nucleotide position
    and may or may not be "indexed"; an "index" is a supplementary file to the
    SAM file that allows for the computer to quickly access various information
    from the alignment for viewing in, for example, the
    [Integrative Genomics Viewer (IGV)](https://igv.org/).

    This step specifically **requires** the BAM files to be sorted an indexed.
    This is not always strictly necessary for downstream tools, but it can be
    convenient to have these files on hand (and BAM files are much smaller than
    uncompressed SAM files, too)."""

    path: str

    type: str


@Function(
    "reads.qc = false",
    "ret.trimmed = reads.trimmed",
    "ret.qc = true",
    "ret.long = reads.long",
    "ret.type = reads.type",
    citation="Simon Andrews. FastQC: a quality control tool for high "
    "throughput sequence data. (2010). Available online at: "
    "http://www.bioinformatics.babraham.ac.uk/projects/fastqc",
    use="a widely-used quality control tool for high throughput sequence data.",
)
def fastqc(__hb_reads: SeqReads, __hb_ret: SeqReads):
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

    fastqs = " ".join(glob.glob(f"{__hb_reads.path}/*.fastq*"))

    __hb_bash(f"""fastqc -t {FASTQC_CORES} -o {__hb_ret.path} {fastqs}""")


@Function(
    "reads.qc = false",
    "ret.trimmed = reads.trimmed",
    "ret.qc = true",
    "ret.long = reads.long",
    "ret.type = reads.type",
    google_scholar_id="15898044054356823756",
    pmid="27312411",
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
def multiqc(__hb_reads: SeqReads, __hb_ret: SeqReads):
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

    fastqs = " ".join(glob.glob(f"{__hb_reads.path}/*.fastq*"))

    __hb_bash(f"""fastqc -t {FASTQC_CORES} -o {__hb_ret.path} {fastqs}""")
    __hb_bash(
        f"""uv run multiqc --filename {__hb_ret.path}/multiqc.html {__hb_ret.path}"""
    )


@Function(
    "reads.qc = true",
    "reads.trimmed = false",
    "ret.qc = false",
    "ret.trimmed = true",
    "ret.long = reads.long",
    "ret.type = reads.type",
    use="to skip adapter trimming (because your sequencing provider already did adapter trimming for you)",
    search=False,
)
def skip_trimming(__hb_reads: SeqReads, __hb_ret: SeqReads):
    """Skip adapter trimming

    If the provider of your sequencing results has said that adapter sequences
    have already been removed, then you don't need to run any processing to
    try to remove them again."""

    carry_over(__hb_reads, __hb_ret, file="sample_sheet.csv")


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = true",
    "ret.type = reads.type",
)
def minimap2(__hb_reads: SeqReads, __hb_ret: SeqAlignment):
    """minimap2

    # Align noisy long reads (~10% error rate) to a reference genome with [minimap2](https://lh3.github.io/minimap2/)

    While traditional aligners like [BWA](https://github.com/lh3/bwa) or
    [bowtie2](https://bowtie-bio.sourceforge.net/bowtie2/index.shtml) _can_
    align long reads, minimap2 is a dedicated tool that aligns long reads much
    more quickly."""

    carry_over(__hb_reads, __hb_ret, file="reference")

    for path in glob.glob(f"{__hb_reads.path}/*.fastq"):
        sample_name = os.path.splitext(os.path.basename(path))[0]
        __hb_bash(f"""
            minimap2 -a \\
                -x map-ont \\
                --sam-hit-only \\
                "{__hb_reads.path}/reference/reference.fasta" \\
                "{path}" \\
                > "{__hb_ret.path}/{sample_name}.sam"
        """)


@Function(
    "ret.type = align.type",
    search=False,
)
def bam_sort_index(__hb_align: SeqAlignment, __hb_ret: SortedIndexBAM):
    """Convert to sorted BAM and index

    # Converted uncompressed SAM files to compressed BAM files (and sort and index them)"""

    carry_over(__hb_align, __hb_ret, file="reference")

    for path in glob.glob(f"{__hb_align.path}/*.sam"):
        sample_name = os.path.splitext(os.path.basename(path))[0]
        __hb_bash(f"""
            samtools view -bS "{path}" \
            | samtools sort -o "{__hb_ret.path}/{sample_name}.bam"
        """)
        __hb_bash(f"""
            samtools index "{__hb_ret.path}/{sample_name}.bam"
        """)


################################################################################
# %% RNA-seq analysis


@Input
class LocalRnaSeq:
    "RNA-seq (stored on your own hard drive)"

    sample_sheet: str
    """Path to sample sheet CSV with SRA metadata

    @example:/Users/jlubin/Desktop/MyExperiment/metadata/sample_sheet.csv

    Here is an example CSV file (the headers must match exactly):

    | sample_name | condition | replicate |
    |-------------|-----------|-----------|
    | JPL001_t1   | treated   | 1         |
    | JPL002_t2   | treated   | 2         |
    | JPL003_u1   | untreated | 1         |
    | JPL004_u2   | untreated | 2         |

    The `sample_name` column can contain whatever you'd like, as long as each
    row is unique."""

    path: str
    """Path to the directory containing the RNA-seq data

    @example:/Users/jlubin/Desktop/MyExperiment/raw-fastq-reads/

    This directory should contain files ending with `.fastq` or `.fastq.gz`."""


@Input
class SraRnaSeq:
    "RNA-seq (stored on the Sequence Read Archive)"

    sample_sheet: str
    """Path to sample sheet CSV with SRA metadata

    @example:/Users/jlubin/Desktop/MyExperiment/metadata/sample_sheet.csv

    Here is an example CSV file (the headers must match exactly):

    | sample_name | condition | replicate |
    |-------------|-----------|-----------|
    | SRR34323945 | treated   | 1         |
    | SRR34323944 | treated   | 2         |
    | SRR34323943 | untreated | 1         |
    | SRR34323942 | untreated | 2         |

    **Important:** The `sample_name` column must contain valid SRA "run
    accessions" (SRRs). They are of the form SRR*xxxxxxxx*, where each *x* is
    digit."""


@Output
class TranscriptMatrices:
    """RNA-seq transcript read counts

    The goal of this step is to calculate two transcript-by-sample matrices:
    - One with (estimated) read counts.
    - One with TPM (transcripts-per-million) abundance.

    These matrices are usually computed by using a reference transcriptome
    (coding sequences) rather than a reference genome. Unless your scientific
    question relates specifically to transcript information, these matrices are
    often aggregated into **gene-level** read count and abundance information."""

    path: str


@Output
class BootstrappedTranscriptMatrices:
    """Transcript read counts (and TPM abundance) of RNA-seq samples, with bootstrap estimates

    The goal of this step is to calculate two transcript-by-sample matrices:
    - One with (estimated) read counts.
    - One with TPM (transcripts-per-million) abundance.

    Additionally, bootstrap resampling estimates are included, which allow
    downstream tools like [sleuth](https://pachterlab.github.io/sleuth/) to
    incorporate measurement uncertainty into differential expression analysis."""

    path: str


@Output
class GeneMatrices:
    """RNA-seq gene read counts

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


@Output
class DifferentialGeneExpression:
    """RNA-seq differential gene expression

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
    """@nosuggest:Path to CSV of comparisons to make

    @example:/Users/jlubin/Desktop/MyExperiment/metadata/comparisons.csv

    Here is an example CSV file (the headers must match exactly):

    | control_condition | treatment_condition |
    |-------------------|---------------------|
    | untreated         | treatment1          |
    | untreated         | treatment2          |

    The entries in this table must be **conditions** from the `condition` column
    in the sample sheet CSV. This comparison CSV tells the software which
    conditions you want to compare to each other."""


@Function(
    "ret.qc = false",
    "ret.trimmed = false",
    "ret.long = false",
    "ret.type = 'rna'",
    search=False,
)
def load_sra_rna_seq(__hb_sra: SraRnaSeq, __hb_ret: SeqReads):
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
    "ret.long = false",
    "ret.type = 'rna'",
    search=False,
)
def load_local_rna_seq(__hb_local: LocalRnaSeq, __hb_ret: SeqReads):
    """Load RNA-seq data from hard drive

    # Load raw RNA-seq data already present on your computer

    The raw RNA-seq files are typically in the `.fastq` or `.fastq.gz` file
    format."""

    carry_over(__hb_local, __hb_ret)

    os.symlink(
        src=__hb_local.sample_sheet,
        dst=f"{__hb_ret.path}/sample_sheet.csv",
    )


@Function(
    "reads.qc = true",
    "reads.trimmed = false",
    "reads.type = 'rna'",
    "ret.qc = false",
    "ret.trimmed = true",
    "ret.long = reads.long",
    "ret.type = 'rna'",
    google_scholar_id="4180123542769751602",
    citation="Marcel Martin. Cutadapt removes adapter sequences from "
    "high-throughput sequencing reads. EMBnet.Journal, 17(1):10-12, May 2011. "
    "http://dx.doi.org/10.14806/ej.17.1.200",
    use="to remove the Illumina universal adapter and poly(A)-tails from mRNA",
)
def cutadapt_illumina(__hb_reads: SeqReads, __hb_ret: SeqReads):
    """cutadapt (Illumina + poly(A))

    # Remove Illumina universal adapter for RNA-seq and poly(A) tails using [cutadapt](https://cutadapt.readthedocs.io/en/stable/).

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


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "ret.type = 'rna'",
)
def star(__hb_reads: SeqReads, __hb_ret: SeqAlignment):
    """STAR"""

    # PARAMETER: The location of the STAR index on your computer
    STAR_REFERENCE = "/Users/jlubin/Desktop/Indexes/star_index"

    # PARAMETER: The number of cores that you want STAR to use
    STAR_CORES = 4

    carry_over(__hb_reads, __hb_ret, file="sample_sheet.csv")

    sample_sheet = pl.read_csv(f"{__hb_reads.path}/sample_sheet.csv")

    for sample_name in sample_sheet["sample_name"]:
        __hb_bash(f"""STAR \\
                  --runThreadN {STAR_CORES}
                  --genomeDir {STAR_REFERENCE}
                  --readFilesIn {__hb_reads.path}/{sample_name}_1.fastq.gz {__hb_reads.path}/{sample_name}_2.fastq.gz""")


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "reads.type = 'rna'",
    google_scholar_id="15817796957364212470",
    pmid="27043002",
    citation="NL Bray, H Pimentel, P Melsted and L Pachter, Near optimal "
    "probabilistic RNA-seq quantification, Nature Biotechnology 34, "
    "p 525--527 (2016).",
)
def kallisto(__hb_reads: SeqReads, __hb_ret: TranscriptMatrices):
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


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "reads.type = 'rna'",
    google_scholar_id="15817796957364212470",
    pmid="27043002",
    citation="NL Bray, H Pimentel, P Melsted and L Pachter, Near optimal "
    "probabilistic RNA-seq quantification, Nature Biotechnology 34, "
    "p 525--527 (2016).",
)
def kallisto_bootstrap(__hb_reads: SeqReads, __hb_ret: BootstrappedTranscriptMatrices):
    """kallisto (with bootstrap)

    # Quantify transcript abundances *without* alignment using [kallisto](https://pachterlab.github.io/kallisto/), with bootstrap estimates

    kallisto is a tool that estimates the number of times a transcript appears
    using a technique called _pseudoalignment_ that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s.

    This version runs kallisto with bootstrap resampling, which is required by
    downstream tools like [sleuth](https://pachterlab.github.io/sleuth/) that
    incorporate measurement uncertainty into differential expression analysis."""

    # PARAMETER: The location of the kallisto transcriptome index on your computer
    KALLISTO_INDEX = "ensembl115.Homo_sapiens.GRCh38.cdna.all.kallisto.idx"

    # PARAMETER: The number of cores that you want kallisto to use
    KALLISTO_CORES = 4

    # PARAMETER: The number of bootstrap samples
    KALLISTO_BOOTSTRAPS = 50

    carry_over(__hb_reads, __hb_ret, file="sample_sheet.csv")

    sample_sheet = pl.read_csv(f"{__hb_reads.path}/sample_sheet.csv")

    for sample_name in sample_sheet["sample_name"]:
        __hb_bash(f"""kallisto quant \\
                    -b {KALLISTO_BOOTSTRAPS} \\
                    -t {KALLISTO_CORES} \\
                    -i {KALLISTO_INDEX} \\
                    -o {__hb_ret.path}/{sample_name} \\
                    {__hb_reads.path}/{sample_name}_1.fastq.gz \\
                    {__hb_reads.path}/{sample_name}_2.fastq.gz""")


################################################################################
# %% Gene matrices


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "reads.type = 'rna'",
    google_scholar_id="11462947284863466602",
    pmid="28263959",
    citation="Patro, R., Duggal, G., Love, M. I., Irizarry, R. A., & "
    "Kingsford, C. (2017). Salmon provides fast and bias-aware quantification "
    "of transcript expression. Nature Methods.",
)
def salmon(__hb_reads: SeqReads, __hb_ret: TranscriptMatrices):
    """Salmon

    # Quantify transcript abundances *without* alignment using [Salmon](https://salmon.readthedocs.io/en/latest/index.html)

    Salmon is a tool that estimates the number of times a transcript appears
    using a lightweight mapping technique that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s."""

    # PARAMETER: The location of the Salmon transcriptome index on your computer
    SALMON_INDEX = "salmon_sa_index"

    # PARAMETER: The number of cores that you want Salmon to use
    CORES = 4

    carry_over(__hb_reads, __hb_ret, file="sample_sheet.csv")

    sample_sheet = pl.read_csv(f"{__hb_reads.path}/sample_sheet.csv")

    for sample_name in sample_sheet["sample_name"]:
        __hb_bash(f"""salmon quant \\
                    -i {SALMON_INDEX} \\
                    -l A \\
                    -p {CORES} \\
                    -1 {__hb_reads.path}/{sample_name}_1.fastq.gz \\
                    -2 {__hb_reads.path}/{sample_name}_2.fastq.gz \\
                    -o {__hb_ret.path}/{sample_name}""")

        # Convert Salmon's quant.sf to kallisto's abundance.tsv format
        pl.read_csv(f"{__hb_ret.path}/{sample_name}/quant.sf", separator="\t").select(
            pl.col("Name").alias("target_id"),
            pl.col("Length").alias("length"),
            pl.col("EffectiveLength").alias("eff_length"),
            pl.col("NumReads").alias("est_counts"),
            pl.col("TPM").alias("tpm"),
        ).write_csv(f"{__hb_ret.path}/{sample_name}/abundance.tsv", separator="\t")


@Function(
    google_scholar_id="5898741618830664005",
    pmid="26925227",
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


@Function(
    google_scholar_id="16121678637925818947",
    citation="Love MI, Huber W, Anders S (2014). Moderated estimation of fold change "
    "and dispersion for RNA-seq data with DESeq2. Genome Biology, 15, 550. "
    "doi:10.1186/s13059-014-0550-8.",
    pmid="25516281",
    use="a **widely-used tool** that does **not** give you error bars.",
)
def deseq2(__hb_data: GeneMatrices, __hb_ret: DifferentialGeneExpression):
    """DESeq2

    # Find differentially-expressed protein-coding genes with [DESeq2](https://bioconductor.org/packages/release/bioc/html/DESeq2.html)

    DESeq2 models gene expression using what is called a
    <span class="more-info">
        <span style="display: none">
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
# %% LEMONmethyl-seq


@Input
class LocalLemonSeq:
    "LEMONmethyl-seq"

    path: str
    """Path to the directory containing the LEMONmethyl-seq data

    @example:/Users/jlubin/Desktop/MyExperiment/raw-fastq-reads/

    This directory should contain files ending with `.fastq` or `.fastq.gz`."""

    reference: str
    """Path to the reference genome to align the LEMONmethyl-seq data to

    @example:/Users/jlubin/Desktop/MyExperiment/reference.fasta

    The path should be to a `.fasta` file containing one entry (for the
    reference genome)."""


@Output
class UnconvertedLemonSeq:
    """@intermediate:Unconverted LEMONmethyl-seq

    The goal of this step is to load in the LEMONmethyl-seq reads you want to
    analyze. After this step, we also need to convert (or somehow otherwise
    obtain) an in silico EM-converted reference genome to map these reads to."""

    path: str


@Output
class CalledMethylation:
    """Per-cytosine methylation counts

    The goal of this step is to produce a table of "methylation calls"; that is,
    a table where each row corresponds to a cytosine in the provided reference
    genome, and each column of the table provides information about how often
    that cytosine was methylated in the reads being analyzed.

    This information is useful for downstream plotting or simply to look at the
    table itself to understand patterns of methylation!"""

    path: str


@Function(
    search=False,
)
def load_local_lemon_seq(__hb_local: LocalLemonSeq, __hb_ret: UnconvertedLemonSeq):
    """Load LEMONmethyl-seq data from hard drive

    The raw LEMONmethyl-seq files are typically in the `.fastq` or `.fastq.gz`
    file format."""

    carry_over(__hb_local, __hb_ret)

    save(
        __hb_local.reference,
        f"{__hb_ret.path}/reference/unconverted.fasta",
    )


@Function(
    "ret.qc = false",
    "ret.trimmed = true",
    "ret.long = true",
    "ret.type = 'lemon'",
    use="to make a new EM-converted reference",
    search=False,
)
def sed_in_silico_em(__hb_data: UnconvertedLemonSeq, __hb_ret: SeqReads):
    """EM-convert provided reference

    # (In silico) EM-convert the provided reference

    In order to perform alignment of LEMONmethyl-seq reads, we need a version of
    the reference genome that has undergone _in silico EM conversion_; or, in
    other words, that has all Cs converted to Ts in the `.fasta`
    (computationally).

    _This preprocessing step performs the in silico EM conversion._"""

    carry_over(__hb_data, __hb_ret)

    __hb_bash(f"""
        cat "{__hb_ret.path}/reference/unconverted.fasta" \
            | sed '/^>/s/$/ (in silico C -> T converted)/' \
            | sed '/^[^>]/s/C/T/g' \
            | sed '/^[^>]/s/c/t/g' \
            > "{__hb_ret.path}/reference/reference.fasta"
    """)


@Function(
    "ret.qc = false",
    "ret.trimmed = true",
    "ret.long = true",
    "ret.type = 'lemon'",
    use="to reuse an existing EM-converted reference",
    search=False,
)
def use_existing_em_reference(__hb_data: UnconvertedLemonSeq, __hb_ret: SeqReads):
    """Use existing EM-converted reference

    # Use an existing (in silico) EM-converted reference genome

    In order to perform alignment of LEMONmethyl-seq reads, we need a version of
    the reference genome that has undergone _in silico EM conversion_; or, in
    other words, that has all Cs converted to Ts in the `.fasta`
    (computationally).

    If you already have a reference genome that has undergone in silico EM
    conversion, you don't need to redo that step! After downloading the
    completed script, simply put in the path to where that reference genome is
    stored in the parameter below.

    _Crucially, this genome must have all Cs converted to Ts
    computationally!_"""

    # PARAMETER: The path to the (in silico) EM-converted reference genome
    EM_REFERENCE_PATH = "/Users/jlubin/Documents/genomes/converted.fasta"

    carry_over(__hb_data, __hb_ret)

    save(
        EM_REFERENCE_PATH,
        f"{__hb_ret.path}/reference/reference.fasta",
    )


@Function(
    "bam.type = 'lemon'",
)
def lemon_mc(__hb_bam: SortedIndexBAM, __hb_ret: CalledMethylation):
    """LEMONmC.py

    # Call methylation counts from a BAM file

    LEMONmC.py is a lightweight tool to call methylation sites on a reference
    genome given a set of LEMONmethyl-seq reads aligned to an in silico
    converted version of the reference genome.

    For each C in the reference, if there is a C in the aligned read, we know
    that site was methylated (it was protected in the wet lab EM conversion
    process). If there is a T in the aligned read, we know that the site must
    not have been methylated!

    LEMONmC.py counts up the per-site methylated and unmethylated alignments
    and collects the results into a single table, with one entry per cytosine
    in the reference genome."""

    for path in glob.glob(f"{__hb_bam.path}/*.bam"):
        sample_name = os.path.splitext(os.path.basename(path))[0]
        __hb_bash(f"""
            uv run LEMONmC.py \
                --ref "{__hb_bam.path}/reference/unconverted.fasta" \
                --bam "{path}" \
                --tsv "{__hb_ret.path}/{sample_name}.tsv"
        """)


################################################################################
# %% ATAC-seq analysis


# REFER TO https://nbis-workshop-epigenomics.readthedocs.io/en/latest/content/tutorials/ATACseq/lab-atacseq-bulk.html#

# missing from ^: shifting alignments


@Input
class LocalAtacSeq:
    "ATAC-seq (stored on your own hard drive)"

    sample_sheet: str
    """TODO"""

    path: str
    """TODO"""


@Input
class SraAtacSeq:
    "ATAC-seq (stored on the Sequence Read Archive)"

    sample_sheet: str
    """TODO"""


@Output
class AtacPeaks:
    """TODO"""

    path: str


@Function(
    "ret.qc = false",
    "ret.trimmed = false",
    "ret.long = false",
    "ret.type = 'atac'",
    search=False,
)
def load_sra_atac_seq(__hb_sra: SraAtacSeq, __hb_ret: SeqReads):
    """TODO"""

    raise NotImplementedError


@Function(
    "ret.qc = false",
    "ret.trimmed = false",
    "ret.long = false",
    "ret.type = 'atac'",
    search=False,
)
def load_local_atac_seq(__hb_local: LocalAtacSeq, __hb_ret: SeqReads):
    """TODO"""

    raise NotImplementedError


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "ret.type = reads.type",
)
def bowtie2(__hb_reads: SeqReads, __hb_ret: SeqAlignment):
    """TODO"""

    raise NotImplementedError


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "ret.type = reads.type",
)
def bwa(__hb_reads: SeqReads, __hb_ret: SeqAlignment):
    """TODO"""

    raise NotImplementedError

# low prio - sorted won't work yet because it needs to name-sorted
@Function(
    "bam.type = 'atac'",
)
def genrich(__hb_bam: SortedIndexBAM, __hb_ret: AtacPeaks):
    """TODO"""

    raise NotImplementedError

# check if sorted
@Function(
    "bam.type = 'atac'",
)
def macs3(__hb_bam: SortedIndexBAM, __hb_ret: AtacPeaks):
    """TODO"""

    raise NotImplementedError


################################################################################
# %% Stubs


@Function(
    google_scholar_id="1639708055766929241",
    pmid="28581496",
    citation="Harold J. Pimentel, Nicolas Bray, Suzette Puente, Páll Melsted "
    "and Lior Pachter, Differential analysis of RNA-Seq incorporating "
    "quantification uncertainty, Nature Methods (2017), advanced access "
    "http://dx.doi.org/10.1038/nmeth.4324.",
    use="a **lesser-used (but still very common)** tool that **does** give you error bars.",
)
def sleuth(
    __hb_data: BootstrappedTranscriptMatrices, __hb_ret: DifferentialGeneExpression
):
    """sleuth

    # Find differentially-expressed protein-coding genes with [sleuth](https://pachterlab.github.io/sleuth/)

    sleuth can find differentially-expressed genes between samples and can
    incorporate measurements of uncertainty using "bootstrap estimates" from
    read quantifiers like [kallisto](http://pachterlab.github.io/kallisto).
    sleuth also has a collection of [walkthroughs](https://pachterlab.github.io/sleuth/walkthroughs)
    that demonstrate how to use it to analyze RNA-seq datasets."""

    # PARAMETER: The version of Ensembl to use for gene annotations
    ENSEMBL_VERSION = "115"

    # PARAMETER: The Ensembl gene annotation dataset to use
    ENSEMBL_DATASET = "hsapiens_gene_ensembl"

    carry_over(__hb_data, __hb_ret, file="sample_sheet.csv")

    __hb_bash(f"""
        Rscript sleuth.r \\
            {ENSEMBL_VERSION} \\
            {ENSEMBL_DATASET} \\
            {__hb_data.path}/sample_sheet.csv \\
            {__hb_ret.comparison_sheet} \\
            {__hb_data.path} \\
            {__hb_ret.path}""")


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
