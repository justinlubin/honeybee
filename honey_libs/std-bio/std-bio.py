import os
import glob
import polars as pl

from honey_lang import Helper, Input, Output, Function, __hb_bash

################################################################################
# %% Helper


@Helper
def stem(path):
    return os.path.splitext(os.path.basename(path))[0]


# @Helper
# def carry_over(src_object, dst_object, *, file=None):
#     def carry_one(f):
#         src = f"../../{src_object.path}/{f}"  # relative to dst
#         dst = f"{dst_object.path}/{f}"
#         if os.path.islink(src):
#             src = os.readlink(src)
#         os.symlink(src=src, dst=dst)

#     if file is None:
#         for file in os.listdir(src_object.path):
#             carry_one(file)
#     else:
#         carry_one(file)


@Helper
def carry_over(source, destination):
    if "*" in source:
        for filename in glob.glob(source):
            src = filename
            if os.path.islink(src):
                src = os.readlink(src)
            src = os.path.relpath(src, start=destination)
            dst = f"{destination}/{filename}"
            os.symlink(src=src, dst=dst)
    elif os.path.isdir(source):
        for filename in os.listdir(source):
            src = filename
            if os.path.islink(src):
                src = os.readlink(src)
            src = os.path.relpath(src, start=destination)
            dst = f"{destination}/{filename}"
            os.symlink(src=src, dst=dst)
    else:
        if os.path.islink(source):
            source = os.readlink(source)
        source = os.path.relpath(
            source,
            start=os.path.dirname(destination),
        )
        os.symlink(src=source, dst=destination)


@Helper
def save(src, dst):
    dir = os.path.dirname(dst)
    os.makedirs(dir, exist_ok=True)
    os.symlink(src=os.path.abspath(src), dst=dst)


@Helper
def shared():
    return "output/shared"


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
    """Data path

    Directory structure:
    - *.fastq: raw data
    - reference/*: files for the reference for the reads"""

    qc: bool
    trimmed: bool
    long: bool
    type: str


@Output
class SeqAlignment:
    """Sequence alignment

    [SAM files](https://doi.org/10.1093/bioinformatics/btp352) are
    **uncompressed** files that describe an alignment of reads to a reference
    genome. These alignments may or may not be "sorted" by nucleotide position
    and may or may not be "indexed"; an "index" is a supplementary file to the
    SAM file that allows for the computer to quickly access various information
    from the alignment for viewing in, for example, the
    [Integrative Genomics Viewer (IGV)](https://igv.org/).

    The equivalent **compressed** files are BAM files."""

    path: str

    compressed: bool
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

    # PARAMETER: The number of cores to use
    CORES = 4

    carry_over(__hb_reads.path, __hb_ret.path)

    fastqs = " ".join(glob.glob(f"{__hb_reads.path}/*.fastq*"))

    # -t number of cores, -o output
    __hb_bash(f"""fastqc -t {CORES} -o {__hb_ret.path} {fastqs}""")


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

    # PARAMETER: The number of cores to use
    CORES = 4

    carry_over(__hb_reads.path, __hb_ret.path)

    fastqs = " ".join(glob.glob(f"{__hb_reads.path}/*.fastq*"))

    # -t number of cores, -o output
    __hb_bash(f"""fastqc -t {CORES} -o {__hb_ret.path} {fastqs}""")
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

    carry_over(__hb_reads.path, __hb_ret.path)


@Function(
    "reads.qc = true",
    "reads.trimmed = false",
    "ret.qc = false",
    "ret.trimmed = true",
    "ret.long = reads.long",
    "ret.type = reads.type",
    google_scholar_id="4180123542769751602",
    citation="Marcel Martin. Cutadapt removes adapter sequences from "
    "high-throughput sequencing reads. EMBnet.Journal, 17(1):10-12, May 2011. "
    "http://dx.doi.org/10.14806/ej.17.1.200",
    use="to remove sequencing adapters",
)
def cutadapt(__hb_reads: SeqReads, __hb_ret: SeqReads):
    """cutadapt (include poly(A) tails)

    # Remove sequencing adapters using [cutadapt](https://cutadapt.readthedocs.io/en/stable/).

    [Adapter trimming](https://knowledge.illumina.com/library-preparation/general/library-preparation-general-reference_material-list/000001314)
    removes adapter sequences that are present due to a read length being
    longer than the insert size of the sequence in a sequencer."""

    # PARAMETER: The forward adapter, by default the Illumina universal
    FORWARD_ADAPTER = "AGATCGGAAGAGCACACGTCTGAACTCCAGTCA"

    # PARAMETER: The reverse adapter, by default the Illumina universal (can delete for single-end only)
    REVERSE_ADAPTER = "AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT"

    # PARAMETER: The number of cores to use
    CORES = 4

    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    for sample in sample_sheet.rows(named=True):
        # Paired-end
        if sample["reverse_location"]:
            # --cores number of cores, -m 1 remove reads <1, -a forward adapter
            # -A reverse adapter, -o forward output, -o reverse output
            __hb_bash(f"""uv run cutadapt \\
                        --cores={CORES} \\
                        -m 1 \\
                        -a {FORWARD_ADAPTER} \\
                        -A {REVERSE_ADAPTER} \\
                        -o {__hb_ret.path}/{sample["forward_location"]} \\
                        -p {__hb_ret.path}/{sample["reverse_location"]} \\
                        {__hb_reads.path}/{sample["forward_location"]} \\
                        {__hb_reads.path}/{sample["reverse_location"]}""")

        # Single-end
        else:
            # --cores number of cores, -m 1 remove reads <1, -a forward adapter
            # -o forward output
            __hb_bash(f"""uv run cutadapt \\
                        --cores={CORES} \\
                        -m 1 \\
                        -a {FORWARD_ADAPTER} \\
                        -o {__hb_ret.path}/{sample["forward_location"]} \\
                        {__hb_reads.path}/{sample["forward_location"]}""")


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = true",
    "ret.type = reads.type",
    "ret.compressed = false",
    google_scholar_id="5235337231718602932",
    pmid="29750242",
    citation="Li H. Minimap2: pairwise alignment for nucleotide sequences. Bioinformatics. 2018 Sep 15;34(18):3094-3100. doi: 10.1093/bioinformatics/bty191. PMID: 29750242; PMCID: PMC6137996.",
    use="to align long reads",
)
def minimap2(__hb_reads: SeqReads, __hb_ret: SeqAlignment):
    """minimap2

    # Align noisy long reads (~10% error rate) to a reference genome with [minimap2](https://lh3.github.io/minimap2/)

    While traditional aligners like [BWA](https://github.com/lh3/bwa) or
    [bowtie2](https://bowtie-bio.sourceforge.net/bowtie2/index.shtml) _can_
    align long reads, minimap2 is a dedicated tool that aligns long reads much
    more quickly."""

    for path in glob.glob(f"{__hb_reads.path}/*.fastq"):
        sample_name = os.path.splitext(os.path.basename(path))[0]
        __hb_bash(f"""
            minimap2 -a \\
                -x map-ont \\
                --sam-hit-only \\
                "{shared()}/reference/reference.fasta" \\
                "{path}" \\
                > "{__hb_ret.path}/{sample_name}.sam"
        """)


@Function(
    "ret.type = align.type",
    "align.compressed = false",
    "ret.compressed = true",
    search=False,
)
def bam_sort_index(__hb_align: SeqAlignment, __hb_ret: SeqAlignment):
    """Compress to BAM

    # Converted uncompressed SAM files to compressed BAM files

    This step also **sorts** and **indexes** the alignments."""

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
class RnaSeq:
    """RNA-seq

    TODO"""

    sample_sheet: str
    """Path to sample sheet CSV

    @example:/Users/barb/Desktop/MyExperiment/metadata/sample_sheet.csv

    Here is an example CSV file (the headers must match exactly):

    | sample_name | condition | forward_location                | reverse_location                |
    |-------------|-----------|---------------------------------|---------------------------------|
    | BM001_t1    | treated   | /Users/barb/Exp1/t1_R1.fastq.gz | /Users/barb/Exp1/t1_R2.fastq.gz |
    | BM002_t2    | treated   | /Users/barb/Exp1/t2_R1.fastq.gz | /Users/barb/Exp1/t1_R2.fastq.gz |
    | BM003_u1    | untreated | SRR34323943_1                   | SRR3423943_2                    |
    | BM004_u2    | untreated | SRR34323942_1                   | SRR3423942_2                    |

    Each row is one sample. Here is what each column means:
    - **`sample_name`** is a unique identifier for each sample (it can be
      whatever you want as long as it is unique).
    - **`condition`** is the label for the experimental condition for each
      sample (this label can be whatever you want, such as "control" and
      "treatment"). Multiple samples with the same condition are considered
      **biological replicates**.
    - **`forward_location`** is the path to the raw RNA-seq data of forward
      reads, likely ending in `.fastq` or `.fastq.gz`. For paired-end reads,
      the filename is likely to end in something resembling `_1.fastq.gz` or
      `_R1.fastq.gz`. Optionally, you can refer to pre-existing datasets using
      their SRA "run accession" (SRR) identifier. These identifiers are of the
      form SRR*xxxxxxxx*, where each *x* is digit. For both single-end and
      paired-end data, add a _1 to the end of the SRR identifier.
    - **`reverse_location`** (for **paired-end data only**) is the path to the
      reverse reads. This path should be similar to the `forward_location` path
      but have `_2` or `_R2` in the filename instead of `_1` or `_R1`. To refer
      to a pre-existing experiment with an SRR, add `_2` to the end of the SRR
      identifier."""

    comparison_sheet: str
    """(Optional!) Path to comparison sheet CSV

    @example:/Users/barb/Desktop/MyExperiment/metadata/comparison_sheet.csv

    If you want to compare some of the conditions defined in your sample sheet,
    include a path to a CSV that describes the comparisons you want to make
    in this field. Otherwise, you can leave it blank to make _all_ comparisons.
    (Depending on your analysis and number of conditions, it may take a long
    time to run all comparisons!)
    
    Here is an example CSV file (the headers must match exactly):

    | control   | treatment  |
    |-----------|------------|
    | untreated | treatment1 |
    | untreated | treatment2 |

    Each row is one comparison to make.

    The `control` column is the control condition, and the `treatment` column
    is the treatment condition.

    **Important Note:** The entries must match the `condition` names from the
    sample sheet above exactly!"""


@Output
class SalmonIndex:
    """@intermediate:Salmon index

    TODO"""

    path: str


@Output
class KallistoIndex:
    """@intermediate:Kallisto index

    TODO"""

    path: str


@Output
class TranscriptMatrices:
    """RNA-seq transcript read counts

    This step outputs two tables: one with read counts, and one with
    transcripts-per-million (TPM) abundance for each of your samples. Each row
    in these tables is a **gene isoform** (mRNA transcript), of which there may
    be many per gene!

    Additionally, some tools like
    [kallisto](https://pachterlab.github.io/kallisto/about)
    can quantify the uncertainty in read counts, which downstream tools like
    [sleuth](https://pachterlab.github.io/sleuth/)
    can use for plotting error bars.

    ## How these tables are made…

    There are two main ways to create these tables:
    1) **Alignment-based methods** align transcripts to a reference genome in a
       splice-aware fashion; the alignments can then be tallied up.
    2) **Alignment-free methods** directly quantify the transcript abundances
       using a reference transcriptome without actually determining where each
       transcript exactly aligns to.

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

    bootstrapped: bool
    "Quantify uncertainty in results using bootstrap"


@Output
class GeneMatrices:
    """RNA-seq gene read counts

    This step outputs two tables: one with read counts, and one with
    transcripts-per-million (TPM) abundance for each of your samples. Each row
    in these tables is a **single gene**.

    ## How these tables are made…

    There are two main ways to create these tables:
    1) **Alignment-based methods** align transcripts to a reference genome in a
       splice-aware fashion; the alignments can then be tallied up.
    2) **Alignment-free methods** directly quantify the transcript abundances
       using a reference transcriptome without actually determining where each
       transcript exactly aligns to.

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


@Function(
    "ret.qc = false",
    "ret.trimmed = false",
    "ret.long = false",
    "ret.type = 'rna'",
    search=False,
)
def load_rna_seq(__hb_rna: RnaSeq, __hb_ret: SeqReads):
    """Load RNA-seq data from sample sheet

    This code collects all the RNA-seq data defined in the sample sheet. It
    downloads samples identified with SRR accession identifiers from the
    [European Nucleotide Archive](https://www.ebi.ac.uk/ena/browser/home)."""

    sample_sheet = pl.read_csv(__hb_rna.sample_sheet)

    # Store the resulting filenames for the new sample sheet we will create
    new_files = {}

    # Loop through each sample in the sample sheet
    for sample in sample_sheet.rows(named=True):
        # Collect the list of files to symlink/download for this sample
        files = [sample["forward_location"]]
        if sample["reverse_location"]:
            files.append(sample["reverse_location"])

        # Loop through each file
        for file in files:
            # If the file starts with "SRR", download it from ENA
            if file.startswith("SRR"):
                assert file[:-2] in {"_1", "_2"}, "SRR must end in _1 or _2"

                # Construct the URL
                srr = file[:-2]  # remove _1 or _2
                base_url = "ftp://ftp.sra.ebi.ac.uk/vol1/fastq/"
                base_url += srr[:6] + "/"
                base_url += srr[9:].zfill(3) + "/"
                base_url += srr + "/"

                new_file = file + ".fastq.gz"

                # Download the file
                __hb_bash(f"""
                     wget \\
                         --no-clobber \\
                         --directory-prefix={__hb_ret.path} \\
                         {base_url}{new_file}
                """)

            # Otherwise, symlink the file path (on the local machine)
            else:
                new_file = os.path.basename(file)
                os.symlink(
                    src=file,
                    dst=__hb_ret.path + "/" + new_file,
                )
            new_files[file] = new_file

    # Create the new sample sheet with updated filenames
    sample_sheet.with_columns(
        forward_location=pl.col("forward_location").replace(new_files),
        reverse_location=pl.col("reverse_location").replace(new_files),
    ).write_csv(f"{shared()}/sample_sheet.csv")

    # Copy over the comparison sheet, if it exists
    if __hb_rna.comparison_sheet:
        carry_over(
            __hb_rna.comparison_sheet,
            f"{shared()}/comparison_sheet.csv",
        )

    # Otherwise, include all comparisons
    else:
        conditions = sample_sheet["condition"].unique(maintain_order=True)
        controls = []
        treatments = []
        for i in range(0, len(conditions)):
            for j in range(i + 1, len(conditions)):
                controls.append(conditions[i])
                treatments.append(conditions[j])
        pl.DataFrame({"control": controls, "treatment": treatments}).write_csv(
            f"{shared()}/comparison_sheet.csv"
        )


@Function(
    "reads.qc = true",
    "reads.trimmed = false",
    "reads.type = 'rna'",
    "ret.qc = false",
    "ret.trimmed = true",
    "ret.long = reads.long",
    "ret.type = reads.type",
    google_scholar_id="4180123542769751602",
    citation="Marcel Martin. Cutadapt removes adapter sequences from "
    "high-throughput sequencing reads. EMBnet.Journal, 17(1):10-12, May 2011. "
    "http://dx.doi.org/10.14806/ej.17.1.200",
    use="to remove the Illumina universal adapter and poly(A)-tails from mRNA",
)
def cutadapt_rna(__hb_reads: SeqReads, __hb_ret: SeqReads):
    """cutadapt (remove poly(A) tails)

    # Remove sequencing adapters and poly(A) tails using [cutadapt](https://cutadapt.readthedocs.io/en/stable/).

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

    # PARAMETER: The forward adapter, by default the Illumina universal
    FORWARD_ADAPTER = "AGATCGGAAGAGCACACGTCTGAACTCCAGTCA"

    # PARAMETER: The reverse adapter, by default the Illumina universal (can delete for single-end only)
    REVERSE_ADAPTER = "AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT"

    # PARAMETER: The number of cores to use
    CORES = 4

    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    for sample in sample_sheet.rows(named=True):
        # Paired-end
        if sample["reverse_location"]:
            # --cores number of cores, -m 1 remove reads <1, -a forward adapter
            # -A reverse adapter, -o forward output, -o reverse output
            __hb_bash(f"""uv run cutadapt \\
                        --cores={CORES} \\
                        -m 1 \\
                        --poly-a \\
                        -a {FORWARD_ADAPTER} \\
                        -A {REVERSE_ADAPTER} \\
                        -o {__hb_ret.path}/{sample["forward_location"]} \\
                        -p {__hb_ret.path}/{sample["reverse_location"]} \\
                        {__hb_reads.path}/{sample["forward_location"]} \\
                        {__hb_reads.path}/{sample["reverse_location"]}""")

        # Single-end
        else:
            # --cores number of cores, -m 1 remove reads <1, -a forward adapter
            # -o forward output
            __hb_bash(f"""uv run cutadapt \\
                        --cores={CORES} \\
                        -m 1 \\
                        --poly-a \\
                        -a {FORWARD_ADAPTER} \\
                        -o {__hb_ret.path}/{sample["forward_location"]} \\
                        {__hb_reads.path}/{sample["forward_location"]}""")


# @Function(
#     "reads.qc = true",
#     "reads.trimmed = true",
#     "reads.long = false",
#     "reads.type = 'rna'",
#     "ret.type = reads.type",
#     "ret.compressed = false",
# )
# def star(__hb_reads: SeqReads, __hb_ret: SeqAlignment):
#     """STAR"""

#     # PARAMETER: The location of the STAR index on your computer
#     STAR_REFERENCE = "/Users/barb/Desktop/Indexes/star_index"

#     # PARAMETER: The number of cores that you want STAR to use
#     STAR_CORES = 4

#     carry_over(__hb_reads, __hb_ret, file="sample_sheet.csv")

#     sample_sheet = pl.read_csv(f"{__hb_reads.path}/sample_sheet.csv")

#     for sample_name in sample_sheet["sample_name"]:
#         __hb_bash(f"""STAR \\
#                   --runThreadN {STAR_CORES}
#                   --genomeDir {STAR_REFERENCE}
#                   --readFilesIn {__hb_reads.path}/{sample_name}_1.fastq.gz {__hb_reads.path}/{sample_name}_2.fastq.gz""")


@Function
def use_existing_kallisto_index(__hb_ret: KallistoIndex):
    """Use existing index

    TODO"""

    # PARAMETER: The location of the kallisto transcriptome index on your computer
    KALLISTO_INDEX = "ensembl115.Homo_sapiens.GRCh38.cdna.all.kallisto.idx"

    carry_over(
        KALLISTO_INDEX,
        f"{__hb_ret.path}/kallisto.idx",
    )


@Function
def create_kallisto_index(__hb_ret: KallistoIndex):
    """Create new index from transcriptome

    TODO"""

    # PARAMETER: The number of cores to use
    CORES = 4

    # PARAMETER: The location of the reference transcriptome on your computer (FASTA format)
    TRANSCRIPTOME_PATH = "ensembl115.Homo_sapiens.GRCh38.cdna.all.fa.gz"

    # -t number of cores, -i output filename for index
    __hb_bash(f"""
        kallisto index \\
            -t {CORES} \\
            -i {__hb_ret.path}/kallisto.idx \\
            {TRANSCRIPTOME_PATH}
    """)


@Function
def use_existing_salmon_index(__hb_ret: SalmonIndex):
    """Use existing index

    TODO"""

    # PARAMETER: The location of the kallisto transcriptome index on your computer
    SALMON_INDEX = "salmon_index"

    carry_over(
        SALMON_INDEX,
        f"{__hb_ret.path}/salmon_index",
    )


@Function
def create_salmon_index(__hb_ret: SalmonIndex):
    """Create new index from transcriptome

    TODO"""

    # PARAMETER: The number of cores to use
    CORES = 4

    # PARAMETER: The location of the reference transcriptome on your computer (FASTA format)
    TRANSCRIPTOME_PATH = "ensembl115.Homo_sapiens.GRCh38.cdna.all.fa.gz"

    # -p number of cores, -t path to transcriptome, -i output filename for index
    __hb_bash(f"""
        salmon index \\
            -p {CORES} \\
            -t {TRANSCRIPTOME_PATH} \\
            -i {__hb_ret.path}/salmon_index
    """)


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "reads.type = 'rna'",
    "ret.bootstrapped = false",
    google_scholar_id="15817796957364212470",
    pmid="27043002",
    citation="NL Bray, H Pimentel, P Melsted and L Pachter, Near optimal "
    "probabilistic RNA-seq quantification, Nature Biotechnology 34, "
    "p 525--527 (2016).",
)
def kallisto(
    __hb_idx: KallistoIndex,
    __hb_reads: SeqReads,
    __hb_ret: TranscriptMatrices,
):
    """kallisto

    # Quantify transcript abundances *without* alignment using [kallisto](https://pachterlab.github.io/kallisto/)

    kallisto is a tool that estimates the number of times a transcript appears
    using a technique called _pseudoalignment_ that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s."""

    # PARAMETER: The number of cores to use
    CORES = 4

    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    for sample in sample_sheet.rows(named=True):
        # Paired-end
        if sample["reverse_location"]:
            # -t number of cores, -i kallisto index, -o output folder
            __hb_bash(f"""kallisto quant \\
                        -t {CORES} \\
                        -i {__hb_idx.path} \\
                        -o {__hb_ret.path}/{sample["sample_name"]} \\
                        {__hb_reads.path}/{sample["forward_location"]} \\
                        {__hb_reads.path}/{sample["reverse_location"]}""")

        # Single-end
        else:
            raise NotImplementedError


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "reads.type = 'rna'",
    "ret.bootstrapped = true",
    google_scholar_id="15817796957364212470",
    pmid="27043002",
    citation="NL Bray, H Pimentel, P Melsted and L Pachter, Near optimal "
    "probabilistic RNA-seq quantification, Nature Biotechnology 34, "
    "p 525--527 (2016).",
)
def kallisto_bootstrap(
    __hb_idx: KallistoIndex,
    __hb_reads: SeqReads,
    __hb_ret: TranscriptMatrices,
):
    """kallisto (with bootstrap resampling)

    # Quantify transcript abundances *without* alignment using [kallisto](https://pachterlab.github.io/kallisto/)

    kallisto is a tool that estimates the number of times a transcript appears
    using a technique called _pseudoalignment_ that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s.

    This version runs kallisto with bootstrap resampling, which is required by
    downstream tools like [sleuth](https://pachterlab.github.io/sleuth/) that
    incorporate measurement uncertainty into differential expression analysis.

    By default, this code uses 50 bootstrap resmamples. The higher this number
    is, the more precise the quantification of uncertainty is, but the longer
    the code will take to run. So, you should make this number as high as you
    are willing to wait for!"""

    # PARAMETER: The number of cores to use
    CORES = 4

    # PARAMETER: The number of bootstrap resamplings kallisto should perform
    KALLISTO_BOOTSTRAPS = 50

    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    for sample in sample_sheet.rows(named=True):
        # Paired-end
        if sample["reverse_location"]:
            # -b number of bootstrap resamplings, -t number of cores,
            # -i kallisto index, -o output folder
            __hb_bash(f"""kallisto quant \\
                        -b {KALLISTO_BOOTSTRAPS}
                        -t {CORES} \\
                        -i {__hb_idx.path} \\
                        -o {__hb_ret.path}/{sample["sample_name"]} \\
                        {__hb_reads.path}/{sample["forward_location"]} \\
                        {__hb_reads.path}/{sample["reverse_location"]}""")

        # Singe-end
        else:
            raise NotImplementedError


################################################################################
# %% Gene matrices


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "reads.type = 'rna'",
    "ret.bootstrapped = false",
    google_scholar_id="11462947284863466602",
    pmid="28263959",
    citation="Patro, R., Duggal, G., Love, M. I., Irizarry, R. A., & "
    "Kingsford, C. (2017). Salmon provides fast and bias-aware quantification "
    "of transcript expression. Nature Methods.",
)
def salmon(
    __hb_idx: SalmonIndex,
    __hb_reads: SeqReads,
    __hb_ret: TranscriptMatrices,
):
    """Salmon

    # Quantify transcript abundances *without* alignment using [Salmon](https://salmon.readthedocs.io/en/latest/index.html)

    Salmon is a tool that estimates the number of times a transcript appears
    using a lightweight mapping technique that is much faster than a full
    alignment procedure like [STAR](https://github.com/alexdobin/STAR)'s."""

    # PARAMETER: The number of cores to use
    CORES = 4

    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    for sample in sample_sheet["sample_name"]:
        # -p number of cores, -i salmon index, -1 forward reads, -2 reverse
        # reads, -o output folder
        __hb_bash(f"""salmon quant \\
                    -p {CORES} \\
                    -i {__hb_idx.path} \\
                    -1 {__hb_reads.path}/{sample["forward_location"]} \\
                    -2 {__hb_reads.path}/{sample["reverse_location"]} \\
                    -o {__hb_ret.path}/{sample["sample_name"]}""")

        # Convert Salmon's quant.sf to kallisto's abundance.tsv format for
        # compatability
        pl.read_csv(
            f"{__hb_ret.path}/{sample['sample_name']}/quant.sf",
            separator="\t",
        ).select(
            pl.col("Name").alias("target_id"),
            pl.col("Length").alias("length"),
            pl.col("EffectiveLength").alias("eff_length"),
            pl.col("NumReads").alias("est_counts"),
            pl.col("TPM").alias("tpm"),
        ).write_csv(
            f"{__hb_ret.path}/{sample['sample_name']}/abundance.tsv",
            separator="\t",
        )


@Function(
    "data.bootstrapped = false",
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
    same gene together for gene-level downstream analysis."""

    # PARAMETER: The version of Ensembl to use for gene annotations
    ENSEMBL_VERSION = "115"

    # PARAMETER: The Ensembl gene annotation dataset to use
    ENSEMBL_DATASET = "hsapiens_gene_ensembl"

    __hb_bash(f"""
        Rscript tximport.r \\
            {ENSEMBL_VERSION} \\
            {ENSEMBL_DATASET} \\
            {shared()}/sample_sheet.csv \\
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

    __hb_bash(f"""
        Rscript deseq2.r \\
            {ENSEMBL_VERSION} \\
            {ENSEMBL_DATASET} \\
            {shared()}/sample_sheet.csv \\
            {shared()}/comparison_sheet.csv \\
            {__hb_data.path}/counts.csv \\
            {__hb_ret.path}""")


@Function(
    "data.bootstrapped = true",
    google_scholar_id="1639708055766929241",
    pmid="28581496",
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

    # PARAMETER: The version of Ensembl to use for gene annotations
    ENSEMBL_VERSION = "115"

    # PARAMETER: The Ensembl gene annotation dataset to use
    ENSEMBL_DATASET = "hsapiens_gene_ensembl"

    carry_over(
        f"{__hb_data.path}/*.csv",
        __hb_ret.path,
    )

    __hb_bash(f"""
        Rscript sleuth.r \\
            {ENSEMBL_VERSION} \\
            {ENSEMBL_DATASET} \\
            {shared()}/sample_sheet.csv \\
            {shared()}/comparison_sheet.csv \\
            {__hb_data.path} \\
            {__hb_ret.path}""")


################################################################################
# %% LEMONmethyl-seq


@Input
class LocalLemonSeq:
    "LEMONmethyl-seq"

    path: str
    """Path to the directory containing the LEMONmethyl-seq data

    @example:/Users/barb/Desktop/MyExperiment/raw-fastq-reads/

    This directory should contain files ending with `.fastq` or `.fastq.gz`."""

    reference: str
    """Path to the reference genome to align the LEMONmethyl-seq data to

    @example:/Users/barb/Desktop/MyExperiment/reference.fasta

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
class MethylationCalls:
    """Methylation calls (per-cytosine methylation)

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

    carry_over(__hb_local.path, __hb_ret.path)

    save(
        __hb_local.reference,
        f"{shared()}/reference/unconverted.fasta",
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

    carry_over(__hb_data.path, __hb_ret.path)

    __hb_bash(f"""
        cat "{shared()}/reference/unconverted.fasta" \
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
    EM_REFERENCE_PATH = "/Users/barb/Documents/genomes/converted.fasta"

    carry_over(__hb_data.path, __hb_ret.path)

    save(
        EM_REFERENCE_PATH,
        f"{shared()}/reference/reference.fasta",
    )


@Function(
    "bam.type = 'lemon'",
    "bam.compressed = true",
)
def lemon_mc(__hb_bam: SeqAlignment, __hb_ret: MethylationCalls):
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
                --ref "{shared()}/reference/unconverted.fasta" \
                --bam "{path}" \
                --tsv "{__hb_ret.path}/{sample_name}.tsv"
        """)


################################################################################
# %% EM-seq


@Input
class LocalEmSeq:
    "EM-seq"

    path: str
    """Path to the directory containing the raw EM-seq data

    @example:/Users/barb/Desktop/MyExperiment/raw-fastq-reads/

    This directory should contain files ending with `.fastq` or `.fastq.gz`."""


@Output
class EmSeqNoRef:
    "@intermediate:EM-seq data (without converted reference)"

    path: str
    """Data path

    Directory structure:
    - *.fastq: raw EM-seq data
    """


@Function(
    search=False,
)
def load_local_em_seq(__hb_local: LocalEmSeq, __hb_ret: EmSeqNoRef):
    """Load EM-seq data from hard drive

    # Load raw EM-seq data already present on your computer

    The raw EM-seq files are typically in the `.fastq` or `.fastq.gz` file
    format."""

    carry_over(__hb_local.path, __hb_ret.path)


@Function(
    "ret.qc = false",
    "ret.trimmed = false",
    "ret.long = false",
    "ret.type = 'em'",
    use="to make a new Bismark-converted reference",
    search=False,
)
def bismark_genome_preparation(__hb_input: EmSeqNoRef, __hb_ret: SeqReads):
    """Bismark EM-convert reference

    # (In silico) EM-convert the provided reference with [Bismark](https://felixkrueger.github.io/Bismark/bismark/genome_preparation/)

    In order to perform alignment of EM-seq reads, we need a version of
    the reference genome that has undergone _in silico EM conversion_; or, in
    other words, that has all Cs converted to Ts (and Gs to As in the reverse
    reads) in the `.fasta` files (computationally).

    _This preprocessing step performs the in silico EM conversion._"""

    # PARAMETER: The folder containing the (unconverted) reference genome to align against
    REFERENCE_GENOME_FOLDER = "/Users/barb/Documents/genomes/genome_folder"

    __hb_bash(f"""
        bismark_genome_preparation \
            --verbose \
            --parallel 1 \
            {REFERENCE_GENOME_FOLDER}
    """)

    carry_over(__hb_input.path, __hb_ret.path)

    save(
        REFERENCE_GENOME_FOLDER,
        f"{__hb_ret.path}/reference",
    )


@Function(
    "ret.qc = false",
    "ret.trimmed = false",
    "ret.long = false",
    "ret.type = 'em'",
    use="to reuse an existing EM-converted reference",
    search=False,
)
def use_existing_bismark_reference(__hb_input: EmSeqNoRef, __hb_ret: SeqReads):
    """Use existing EM-converted reference

    # Use an existing (in silico) [EM-converted Bismark reference genome](https://felixkrueger.github.io/Bismark/bismark/genome_preparation/)

    In order to perform alignment of EM-seq reads, we need a version of
    the reference genome that has undergone _in silico EM conversion_; or, in
    other words, that has all Cs converted to Ts (and Gs to As in the reverse
    reads) in the `.fasta` files (computationally).

    If you already have a reference genome that has undergone in silico EM
    conversion for Bismark, you don't need to redo that step! After downloading
    the completed script, simply put in the path to where that reference genome
    is stored in the parameter below."""

    # PARAMETER: The folder containing the Bismark reference genome to align against
    BISMARK_GENOME_FOLDER = "/Users/barb/Documents/genomes/bismark_genome_folder"

    carry_over(__hb_input.path, __hb_ret.path)

    save(
        BISMARK_GENOME_FOLDER,
        f"{__hb_ret.path}/reference",
    )


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "reads.type = 'em'",
    "ret.type = reads.type",
    "ret.compressed = true",
    google_scholar_id="10716431004487472020",
    pmid="21493656",
    citation="Krueger F, Andrews SR. Bismark: a flexible aligner and methylation caller for Bisulfite-Seq applications. Bioinformatics. 2011 Jun 1;27(11):1571-2. doi: 10.1093/bioinformatics/btr167. Epub 2011 Apr 14. PMID: 21493656; PMCID: PMC3102221.",
    additional_citations=[
        "Langmead B, Salzberg SL. Fast gapped-read alignment with Bowtie 2. Nat Methods. 2012 Mar 4;9(4):357-9. doi: 10.1038/nmeth.1923. PMID: 22388286; PMCID: PMC3322381."
    ],
)
def bismark(__hb_reads: SeqReads, __hb_ret: SeqAlignment):
    """Bismark

    # Align EM-converted reads with [Bismark](https://felixkrueger.github.io/Bismark/bismark/alignment/)

    Bismark uses [bowtie2](https://bowtie-bio.sourceforge.net/bowtie2/index.shtml)
    under the hood to align wet-lab EM-converted sample reads to an in silico
    EM-converted reference."""

    # PARAMETER: The suffix at the end of the filenames for the forward reads
    FORWARD_READ_SUFFIX = "_R1"

    # PARAMETER: The suffix at the end of the filenames for the reverse reads
    REVERSE_READ_SUFFIX = "_R2"

    for path in glob.glob(f"{__hb_reads.path}/*{FORWARD_READ_SUFFIX}.fastq.gz"):
        sample_name = stem(path).removesuffix(FORWARD_READ_SUFFIX)
        print(f"Running bismark on '{path}'...")
        __hb_bash(f"""
            bismark \
                --bam \
                --parallel 1 \
                --genome {__hb_reads.path}/reference \
                -o {__hb_ret.path} \
                -1 {__hb_reads.path}/{sample_name}{FORWARD_READ_SUFFIX}.fastq.gz \
                -2 {__hb_reads.path}/{sample_name}{REVERSE_READ_SUFFIX}.fastq.gz
        """)


@Function(
    "input.type = 'em'",
    "input.compressed = true",
)
def bismark_methylation_extractor(
    __hb_input: SeqAlignment,
    __hb_ret: MethylationCalls,
):
    """Bismark Methylation Extractor

    # Call methylation with the [Bismark Methylation Extractor](https://felixkrueger.github.io/Bismark/bismark/methylation_extraction/)

    This step also produces bedGraph files and "coverage" files using
    [bismark2bedGraph](https://felixkrueger.github.io/Bismark/bismark/methylation_extraction/#optional-bedgraph-output).

    The most important output files are the **coverage** files, as they contain
    the number of methylated and unmethylated reads at each CpG. These files
    enable essentially any downstream analysis of interest."""

    for path in glob.glob(f"{__hb_input.path}/*.bam"):
        print(f"Running bismark_methylation_extractor on '{path}'...")
        __hb_bash(f"""
            bismark_methylation_extractor \
               --parallel 1 \
               --gzip \
               --bedGraph \
               -o {__hb_ret.path} \
               {path}
        """)


# # PARAMETER: The suffix at the end of the filenames for the forward reads
# FORWARD_READ_SUFFIX = "_R1"

# # Original top (OT) strand
# for path in glob.glob(f"{__hb_ret.path}/CpG_OT_*.txt.gz"):
#     sample_name = stem(path).removesuffix(FORWARD_READ_SUFFIX + "_bismark_bt2_pe")
#     print(f"Running bismark2bedGraph on '{path}' OT...")
#     __hb_bash(f"""
#         bismark2bedGraph \
#            --dir {__hb_ret.path} \
#            --parallel {max(BISMARK_CORES // 3, 1)} \
#            -o {sample_name}_OT.txt \
#            {path}
#     """)

# # Original bottom (OB) strand
# for path in glob.glob(f"{__hb_ret.path}/CpG_OB_*.txt.gz"):
#     sample_name = stem(path).removesuffix(FORWARD_READ_SUFFIX + "_bismark_bt2_pe")
#     print(f"Running bismark2bedGraph on '{path}' OB...")
#     __hb_bash(f"""
#         bismark2bedGraph \
#            --dir {__hb_ret.path} \
#            --parallel {max(BISMARK_CORES // 3, 1)} \
#            -o {sample_name}_OB.txt \
#            {path}
#     """)


################################################################################
# %% ATAC-seq analysis


# REFER TO https://nbis-workshop-epigenomics.readthedocs.io/en/latest/content/tutorials/ATACseq/lab-atacseq-bulk.html#

# missing from ^: shifting alignments


@Input
class AtacSeq:
    "ATAC-seq (stored on your own hard drive)"

    path: str
    """TODO"""

    reference: str
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
def load_local_atac_seq(__hb_local: AtacSeq, __hb_ret: SeqReads):
    """TODO"""

    # symlink on fastqc files and path to reference
    # When loading data, save the reference sheet symlinked to reference/reference.fasta
    carry_over(__hb_local.path, __hb_ret.path)

    save(
        __hb_local.reference,
        f"{__hb_ret.path}/reference/reference.fasta",
    )


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "ret.type = reads.type",
    "ret.compressed = false",
)
def bowtie2(__hb_reads: SeqReads, __hb_ret: SeqAlignment):
    """TODO"""

    carry_over(__hb_reads, __hb_ret, file="reference")

    index_path = f"{__hb_ret.path}/index"

    __hb_bash(f"""
        bowtie2-build \
            -f {__hb_reads.path}/reference/reference.fasta \
            {index_path} \
            > out.txt
    """)

    # get lists of mate1 and mate2 files
    mate1 = []
    mate2 = []
    for path in glob.glob(f"{__hb_reads.path}/*.fastq*"):
        sample_name = os.path.splitext(os.path.basename(path))[0]
        if sample_name[-9:] == "_R1.fastq":
            mate1.append(path)
        elif sample_name[-9:] == "_R2.fastq":
            mate2.append(path)

    # make sure mate1 and mate2 are same len, names match in pairs, are sorted to be at same index
    for i in range(len(mate1)):
        m1 = mate1[i]
        m2 = mate2[i]
        lastslash = m1.rfind("/")
        if lastslash == -1:
            lastslash = 0
        name = m1[lastslash:-12]

        __hb_bash(f"""
            bowtie2 \
                -x "{index_path}" \
                -1 "{m1}" \
                -2 "{m2}" \
                -S "{__hb_ret.path}/{name}.sam"
        """)


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "ret.type = reads.type",
    "ret.compressed = false",
)
def bwa(__hb_reads: SeqReads, __hb_ret: SeqAlignment):
    """TODO"""

    carry_over(__hb_reads, __hb_ret, file="reference")

    ref = f"{__hb_ret.path}/reference/reference.fasta"

    __hb_bash(f"""
        bwa index "{ref}"
    """)

    # get lists of mate1 and mate2 files
    mate1 = []
    mate2 = []
    for path in glob.glob(f"{__hb_reads.path}/*.fastq*"):
        sample_name = os.path.splitext(os.path.basename(path))[0]
        if sample_name[-9:] == "_R1.fastq":
            mate1.append(path)
        elif sample_name[-9:] == "_R2.fastq":
            mate2.append(path)

    # make sure mate1 and mate2 are same len, names match in pairs, are sorted to be at same index
    for i in range(len(mate1)):
        m1 = mate1[i]
        m2 = mate2[i]
        lastslash = m1.rfind("/")
        if lastslash == -1:
            lastslash = 0
        name = m1[lastslash:-12]

        s1 = f"{__hb_reads.path}/{name}_R1.sai"
        s2 = f"{__hb_reads.path}/{name}_R2.sai"

        __hb_bash(f"""
            bwa aln "{ref}" "{m1}" > "{s1}"
        """)

        __hb_bash(f"""
            bwa aln "{ref}" "{m2}" > "{s2}"
        """)

        __hb_bash(f"""
            bwa sampe "{ref}" "{s1}" "{s2}" "{m1}" "{m2}" > "{__hb_ret.path}/{name}.sam"
        """)


@Function(
    "bam.compressed = true",
    "bam.type = 'atac'",
)
def macs3(__hb_bam: SeqAlignment, __hb_ret: AtacPeaks):
    """TODO"""

    # PARAMETER: Effective genome size (use hs for human, mm for mouse, or a number)
    GENOME_SIZE = "hs"

    for path in glob.glob(f"{__hb_bam.path}/*.bam"):
        sample_name = os.path.splitext(os.path.basename(path))[0]
        __hb_bash(f"""
            macs3 callpeak \\
                -t "{path}" \\
                -f BAMPE \\
                -g {GENOME_SIZE} \\
                --nomodel \\
                --keep-dup all \\
                -n {sample_name} \\
                --outdir "{__hb_ret.path}"
        """)

        # Convert xls output (tab-separated with comment headers) to .csv
        xls_path = f"{__hb_ret.path}/{sample_name}_peaks.xls"
        pl.read_csv(xls_path, separator="\t", comment_prefix="#").write_csv(
            f"{__hb_ret.path}/{sample_name}_peaks.csv"
        )


################################################################################
# %% Stubs


# @Function(
#     "ret.label = data.label",
#     "data.bc = false",
#     "ret.bc = true",
# )
# def combat_seq(
#     data: TranscriptMatrices, ret: TranscriptMatrices.S
# ) -> TranscriptMatrices.D:
#     raise NotImplementedError

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
