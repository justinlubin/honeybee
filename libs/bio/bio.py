import glob
import os
import re
import subprocess
import polars as pl
import altair as alt
import sklearn.cluster as skclust
import sklearn.decomposition as skdec
from IPython.display import display

from honey_lang import Function, Helper, Input, Output, initialize, log

initialize(
    name="Honeybee Biology Library Release 8 (<<<COMMIT>>>)",
    allow_input_arguments=False,
)

################################################################################
# %% Helper


@Helper
def bash(command, redirect_stderr=True):
    command = command.replace("\\\n", "\n")
    command = re.sub(r"\s+", " ", command).strip()

    log("### Running bash command:\n")
    log(command + "\n")
    log("### Output:\n")

    with subprocess.Popen(
        command,
        shell=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT if redirect_stderr else None,
        bufsize=1,
    ) as p:
        if p.stdout:
            for line in p.stdout:
                log(line.removesuffix("\n"))

    log(f"\n### Exit code:\n\n{p.returncode}\n")


@Helper
def stem(path):
    ret = path
    while True:
        ret, ext = os.path.splitext(os.path.basename(ret))
        if not ext:
            return ret


@Helper
def carry_over(source_glob, destination_folder):
    os.makedirs(destination_folder, exist_ok=True)
    if "*" not in source_glob:
        source_glob += "/*"
    sources = glob.glob(source_glob)
    assert sources != [], f"Pattern '{source_glob}' did not match any files"
    for src in sorted(sources):
        basename = os.path.basename(src)
        src = os.path.relpath(src, start=destination_folder)
        dst = f"{destination_folder}/{basename}"
        os.symlink(src=src, dst=dst)


@Helper
def link(src, dst):
    assert os.path.exists(src), f"Cannot find file/folder '{src}'"
    destination_folder = os.path.dirname(dst)
    os.makedirs(destination_folder, exist_ok=True)
    src = os.path.relpath(src, start=destination_folder)
    os.symlink(src=src, dst=dst)


@Helper
def shared():
    path = "output/shared"
    os.makedirs(path, exist_ok=True)
    return path


@Helper
def already_exists(path):
    if not os.path.isdir(path):
        return False
    for file in os.listdir(path):
        if not file.startswith("."):
            return True
    return False


@Helper
def needs_to_run(path):
    if already_exists(path):
        print(f"'{path}' already exists, skipping step (delete folder to re-run)")
        return True
    else:
        bash(f"""mkdir -p {path}""")
        return False


################################################################################
# %% Sequencing reads


@Output(
    phase=0,
    phase_description="Preprocessing",
)
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
    as _quality control_ (QC) checks.

    **Note:** Often, you will need to trim "adapter" sequences from your reads;
    these are sequences of nucleotides that are an artifact of the sequencing
    process. It is good practice to run quality control checks **twice**: once
    before trimming, and once after. (In fact, this practice is enforced by
    this interface!)"""

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
    """@intermediate:Sequence alignment

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

    # Carry forward the sequencing data for downstream steps
    carry_over(__hb_reads.path, __hb_ret.path)

    # Get the files to run FastQC on
    fastqs = " ".join(sorted(glob.glob(f"{__hb_reads.path}/*.fastq*")))

    # Run FastQC (-t number of cores, -o output)
    bash(f"""fastqc -t {CORES} -o {__hb_ret.path} {fastqs}""")


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

    # Carry forward the sequencing data for downstream steps
    carry_over(__hb_reads.path, __hb_ret.path)

    # Get the files to run FastQC on
    fastqs = " ".join(sorted(glob.glob(f"{__hb_reads.path}/*.fastq*")))

    # Run FastQC (-t number of cores, -o output)
    bash(f"""fastqc -t {CORES} -o {__hb_ret.path} {fastqs}""")

    # Aggregate FastQC results with MultiQC
    bash(f"""uv run multiqc --filename {__hb_ret.path}/multiqc.html {__hb_ret.path}""")


@Function(
    "reads.qc = false",
    "reads.trimmed = false",
    "ret.qc = false",
    "ret.trimmed = true",
    "ret.long = reads.long",
    "ret.type = reads.type",
    use="to skip adapter trimming (e.g. because your sequencing provider already did adapter trimming for you, or you want to save time)",
    search=False,
)
def skip_trimming(__hb_reads: SeqReads, __hb_ret: SeqReads):
    """Skip adapter trimming

    If the provider of your sequencing results has said that adapter sequences
    have already been removed, then you don't need to run any processing to
    try to remove them again."""

    # Carry forward the sequencing data for downstream steps;
    # do not perform any trimming
    carry_over(__hb_reads.path, __hb_ret.path)


@Function(
    "reads.qc = true",
    "reads.trimmed = false",
    "ret.qc = false",
    "ret.trimmed = true",
    "ret.long = reads.long",
    "ret.type = reads.type",
    "ret.type != 'rna'",
    google_scholar_id="4180123542769751602",
    citation="Marcel Martin. Cutadapt removes adapter sequences from "
    "high-throughput sequencing reads. EMBnet.Journal, 17(1):10-12, May 2011. "
    "http://dx.doi.org/10.14806/ej.17.1.200",
    use="to remove sequencing adapters from DNA",
)
def cutadapt(__hb_reads: SeqReads, __hb_ret: SeqReads):
    """cutadapt

    # Remove sequencing adapters using [cutadapt](https://cutadapt.readthedocs.io/en/stable/)

    [Adapter trimming](https://knowledge.illumina.com/library-preparation/general/library-preparation-general-reference_material-list/000001314)
    removes adapter sequences that are present due to a read length being
    longer than the insert size of the sequence in a sequencer."""

    # PARAMETER: The forward adapter, by default the Illumina universal
    FORWARD_ADAPTER = "AGATCGGAAGAGCACACGTCTGAACTCCAGTCA"

    # PARAMETER: The reverse adapter, by default the Illumina universal (can delete for single-end only)
    REVERSE_ADAPTER = "AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT"

    # PARAMETER: The number of cores to use
    CORES = 4

    # Load the sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Run cutadapt for each sample in the sample sheet
    for sample in sample_sheet.rows(named=True):
        # --cores number of cores, -m 1 remove reads <1, -a forward adapter
        # -A reverse adapter, -o forward output, -o reverse output
        bash(f"""
            uv run cutadapt
                --cores={CORES}
                -m 1
                -a {FORWARD_ADAPTER}
                -A {REVERSE_ADAPTER}
                -o {__hb_ret.path}/{sample["forward_location"]}
                -p {__hb_ret.path}/{sample["reverse_location"]}
                {__hb_reads.path}/{sample["forward_location"]}
                {__hb_reads.path}/{sample["reverse_location"]}i
        """)


# # Single-end reads
# else:
#     # --cores number of cores, -m 1 remove reads <1, -a forward adapter
#     # -o forward output
#     bash(f"""uv run cutadapt
#                 --cores={CORES}
#                 -m 1
#                 -a {FORWARD_ADAPTER}
#                 -o {__hb_ret.path}/{sample["forward_location"]}
#                 {__hb_reads.path}/{sample["forward_location"]}""")


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

    # Run minimap2 for each sample
    for path in sorted(glob.glob(f"{__hb_reads.path}/*.fastq")):
        sample_name = os.path.splitext(os.path.basename(path))[0]
        bash(f"""
            minimap2 -a
                -x map-ont
                --sam-hit-only
                "{shared()}/reference/reference.fasta"
                "{path}"
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

    for path in sorted(glob.glob(f"{__hb_align.path}/*.sam")):
        sample_name = os.path.splitext(os.path.basename(path))[0]
        bash(f"""
            samtools view -bS "{path}" \
            | samtools sort -o "{__hb_ret.path}/{sample_name}.bam"
        """)
        bash(f"""
            samtools index "{__hb_ret.path}/{sample_name}.bam"
        """)


@Output
class Bowtie2Index:
    """@intermediate:Bowtie 2 index

    The goal of this step is to create an index for the
    [bowtie2](https://bowtie-bio.sourceforge.net/bowtie2/index.shtml)
    read aligner."""

    path: str


@Function(
    search=False,
)
def use_existing_bowtie2_index(__hb_ret: Bowtie2Index):
    """Use existing index

    If you already have a Bowtie 2 index for the reference genome you'd like to
    use, you can use this code to skip creating a new one."""

    # PARAMETER: The location of the Bowtie 2 reference genome index on your computer
    BOWTIE2_INDEX = "folder/"

    link(
        BOWTIE2_INDEX,
        f"{__hb_ret.path}/reference",
    )


@Function(
    search=False,
)
def create_bowtie2_index(__hb_ret: Bowtie2Index):
    """New from reference genome

    This code creates a Bowtie 2 index from an existing reference genome file on
    your computer. A good place to download reference genomes is the
    [Ensembl genome repository](https://www.ensembl.org/)."""

    # PARAMETER: The number of cores to use
    CORES = 4

    # PARAMETER: The location of the reference genome on your computer (FASTA format)
    REFERENCE_GENOME_PATH = "Homo_sapiens.GRCh38.dna.primary_assembly.fa.gz"

    bash(f"""mkdir -p {__hb_ret.path}/reference""")

    # --threads number of cores, first argument reference path, second argument
    # output path
    bash(f"""
            bowtie2-build
                --threads {CORES}
                {REFERENCE_GENOME_PATH}
                {__hb_ret.path}/reference/ref
    """)


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "reads.type != 'rna'",
    "ret.type = reads.type",
    "ret.compressed = false",
    google_scholar_id="4636016874467197548",
    pmid="22388286",
    citation="Langmead B, Salzberg SL. Fast gapped-read alignment with Bowtie 2. Nat Methods. 2012 Mar 4;9(4):357-9. doi: 10.1038/nmeth.1923. PMID: 22388286; PMCID: PMC3322381.",
)
def bowtie2(
    __hb_idx: Bowtie2Index,
    __hb_reads: SeqReads,
    __hb_ret: SeqAlignment,
):
    """Bowtie 2

    # Align reads to a reference genome with [Bowtie 2](https://bowtie-bio.sourceforge.net/bowtie2/index.shtml)

    Bowtie 2 aligns DNA reads to a reference genome, which is useful, for
    example, when studying the genome of an organism or doing a metagemoics
    analysis. Some genomics assays that could make excellent use of Bowtie 2
    include ATAC-seq, ChIP-seq, and EM-seq.

    Bowtie 2 should **not** be used when aligning mRNA transcripts to the
    complementary DNA, as it is not splice-aware like
    [STAR](https://github.com/alexdobin/STAR)'s. (Most RNA-seq experiments
    should use a transcript quantifier like
    [kallisto](https://pachterlab.github.io/kallisto/) or
    [Salmon](https://salmon.readthedocs.io/en/latest/index.html).)"""

    # PARAMETER: The number of cores to use
    CORES = 4

    index_name = stem(glob.glob(f"{__hb_idx.path}/reference/*.bt2")[0])

    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    for sample in sample_sheet.rows(named=True):
        # Paired-end
        if sample["reverse_location"]:
            # -x name of reference, -p number of cores, -1 forward reads, -2
            # reverse reads, -S output file
            bash(f"""
                 bowtie2
                     -p {CORES}
                     -x {__hb_idx.path}/reference/{index_name}
                     -1 {__hb_reads.path}/{sample["forward_location"]}
                     -2 {__hb_reads.path}/{sample["reverse_location"]}
                     -S {__hb_ret.path}/{sample["sample_name"]}.sam
            """)

        # Single-end
        else:
            # -x name of reference, -p number of cores, -U unpaired reads,
            # -S output file
            bash(f"""
                 bowtie2
                     -p {CORES}
                     -x {__hb_idx.path}/reference/{index_name}
                     -U {__hb_reads.path}/{sample["forward_location"]}
                     -S {__hb_ret.path}/{sample["sample_name"]}.sam
            """)


@Output
class BWAIndex:
    """@intermediate:BWA index

    The goal of this step is to create an index for the
    [BWA-MEM](https://bio-bwa.sourceforge.net/)
    read aligner."""

    path: str


@Function(
    search=False,
)
def use_existing_bwa_index(__hb_ret: BWAIndex):
    """Use existing index

    If you already have a BWA index for the reference genome you'd like to
    use, you can use this code to skip creating a new one."""

    # PARAMETER: The location of the Bowtie 2 reference genome index on your computer
    BWA_INDEX = "folder/"

    link(
        BWA_INDEX,
        f"{__hb_ret.path}/reference",
    )


@Function(
    search=False,
)
def create_bwa_index(__hb_ret: BWAIndex):
    """New from reference genome

    This code creates a BWA index from an existing reference genome file on
    your computer. A good place to download reference genomes is the
    [Ensembl genome repository](https://www.ensembl.org/)."""

    # PARAMETER: The location of the reference genome on your computer (FASTA format)
    REFERENCE_GENOME_PATH = "Homo_sapiens.GRCh38.dna.primary_assembly.fa.gz"

    bash(f"""mkdir -p {__hb_ret.path}/reference""")

    # -p output path
    bash(f"""
        bwa index
            -p {__hb_ret.path}/reference/ref
            {REFERENCE_GENOME_PATH}
    """)


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "reads.type != 'rna'",
    "ret.type = reads.type",
    "ret.compressed = false",
    google_scholar_id="4636016874467197548",
    pmid="22388286",
    citation="Li H. and Durbin R. (2009) Fast and accurate short read alignment with Burrows-Wheeler Transform. Bioinformatics, 25:1754-60. [PMID: 19451168]",
)
def bwa_mem(
    __hb_idx: BWAIndex,
    __hb_reads: SeqReads,
    __hb_ret: SeqAlignment,
):
    """BWA-MEM

    # Align reads to a reference genome with [BWA-MEM](https://bio-bwa.sourceforge.net/)

    BWA-MEM aligns DNA reads to a reference genome, which is useful, for
    example, when studying the genome of an organism or doing a metagemoics
    analysis. Some genomics assays that could make excellent use of BWA-MEM
    include ATAC-seq, ChIP-seq, and EM-seq.

    BWA-MEM should **not** be used when aligning mRNA transcripts to the
    complementary DNA, as it is not splice-aware like
    [STAR](https://github.com/alexdobin/STAR)'s. (Most RNA-seq experiments
    should use a transcript quantifier like
    [kallisto](https://pachterlab.github.io/kallisto/) or
    [Salmon](https://salmon.readthedocs.io/en/latest/index.html).)"""

    # PARAMETER: The number of cores to use
    CORES = 4

    index_name = stem(glob.glob(f"{__hb_idx.path}/reference/*.amb")[0])

    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    for sample in sample_sheet.rows(named=True):
        # Works for paired-end and single-end

        # -t number of cores
        bash(f"""
             bwa mem
                 -t {CORES}
                 {__hb_idx.path}/reference/{index_name}
                 {__hb_reads.path}/{sample["forward_location"]}
                 {__hb_reads.path}/{sample["reverse_location"]}
                     > {__hb_ret.path}/{sample["sample_name"]}.sam
        """)


################################################################################
# %% RNA-seq analysis

# sample_sheet: str
# """Path to sample sheet CSV

# @example:metadata/sample_sheet.csv

# Here is an example CSV file (the headers must match exactly):

# | sample_name | condition | forward_location        | reverse_location        |
# |-------------|-----------|-------------------------|-------------------------|
# | BM001_t1    | treated   | raw-data/t1_R1.fastq.gz | raw-data/t1_R2.fastq.gz |
# | BM002_t2    | treated   | raw-data/t2_R1.fastq.gz | raw-data/t1_R2.fastq.gz |
# | BM003_u1    | untreated | SRR34323943_1           | SRR3423943_2            |
# | BM004_u2    | untreated | SRR34323942_1           | SRR3423942_2            |

# Each row is one sample. Here is what each column means:
# - **`sample_name`** is a unique identifier for each sample (it can be
#   whatever you want as long as it is unique).
# - **`condition`** is the label for the experimental condition for each
#   sample (this label can be whatever you want, such as "control" and
#   "treatment"). Multiple samples with the same condition are considered
#   **biological replicates**.
# - **`forward_location`** is the path to the raw RNA-seq data of forward
#   reads, likely ending in `.fastq` or `.fastq.gz`. For paired-end reads,
#   the filename is likely to end in something resembling `_1.fastq.gz` or
#   `_R1.fastq.gz`. Optionally, you can refer to pre-existing datasets using
#   their SRA "run accession" (SRR) identifier. These identifiers are of the
#   form SRR*xxxxxxxx*, where each *x* is digit. For both single-end and
#   paired-end data, add a _1 to the end of the SRR identifier.
# - **`reverse_location`** (for **paired-end data only**) is the path to the
#   reverse reads. This path should be similar to the `forward_location` path
#   but have `_2` or `_R2` in the filename instead of `_1` or `_R1`. To refer
#   to a pre-existing experiment with an SRR, add `_2` to the end of the SRR
#   identifier."""

# comparison_sheet: str
# """(Optional!) Path to comparison sheet CSV

# @example:metadata/comparison_sheet.csv

# If you want to compare some of the conditions defined in your sample sheet,
# include a path to a CSV that describes the comparisons you want to make
# in this field. Otherwise, you can leave it blank to make _all_ comparisons.
# (Depending on your analysis and number of conditions, it may take a long
# time to run all comparisons!)

# Here is an example CSV file (the headers must match exactly):

# | control_condition | treatment_condition |
# |-------------------|---------------------|
# | untreated         | treatment1          |
# | untreated         | treatment2          |

# Each row is one comparison to make.

# The `control_condition` column is the control condition, and the
# `treatment_condition` column is the treatment condition.

# **Important Note:** The entries must match the `condition` names from the
# sample sheet above exactly!"""


@Input
class RnaSeq:
    """RNA-seq (bulk)

    Bulk RNA sequencing (RNA-seq) is a technique to sequence the RNA content of cells
    or tissues, commonly done via
    [Illumina sequencing](https://www.illumina.com/techniques/sequencing/rna-sequencing.html)."""


@Output(
    phase=1,
    phase_description="Read Quantification",
)
class SalmonIndex:
    """@intermediate:Salmon index

    The goal of this step is to create an index for the
    [salmon](https://salmon.readthedocs.io/en/latest/) transcript abundance
    quantifier.

    You can think of an _index_ like the index in a book: by sorting all the
    concepts in a book alphabetically (which takes some time to do up-front),
    it's a lot quicker to find where each concept is located in the book! It's
    the same with a genome: Salmon _indexes_ it ahead of time so that it can
    find locations much quicker as it operates."""

    path: str


@Output(
    phase=1,
    phase_description="Read Quantification",
)
class KallistoIndex:
    """@intermediate:Kallisto index

    The goal of this step is to create an index for the
    [kallisto](https://pachterlab.github.io/kallisto/about)
    transcript abundance quantifier.

    You can think of an _index_ like the index in a book: by sorting all the
    concepts in a book alphabetically (which takes some time to do up-front),
    it's a lot quicker to find where each concept is located in the book! It's
    the same with a genome: kallisto _indexes_ it ahead of time so that it can
    find locations much quicker as it operates."""

    path: str


@Output(
    phase=1,
    phase_description="Read Quantification",
)
class StarIndex:
    """@intermediate:STAR index

    The goal of this step is to create an index for the
    [STAR](https://github.com/alexdobin/STAR) splice-aware RNA-seq read
    aligner.

    You can think of an _index_ like the index in a book: by sorting all the
    concepts in a book alphabetically (which takes some time to do up-front),
    it's a lot quicker to find where each concept is located in the book! It's
    the same with a genome: STAR _indexes_ it ahead of time so that it can
    find locations much quicker as it operates."""

    path: str


@Output(
    phase=1,
    phase_description="Read Quantification",
)
class TranscriptMatrices:
    """Tally reads mapping to transcripts

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

    These tables can be used for plotting, differential expression testing,
    clustering, and many other downstream analyses. The following review
    provides an overview of RNA-seq data analysis, including information about
    read count tables (Fig 2a and 2b are especially relevant):

    > Conesa, A., Madrigal, P., Tarazona, S. et al. A survey of best practices
    > for RNA-seq data analysis. Genome Biol 17, 13 (2016).
    > https://doi.org/10.1186/s13059-016-0881-8"""

    path: str

    bootstrapped: bool
    "Quantify uncertainty in results using bootstrap"


@Output(
    phase=1,
    phase_description="Read Quantification",
)
class GeneMatrices:
    """Tally reads mapping to genes

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

    These tables can be used for plotting, differential expression testing,
    clustering, and many other downstream analyses. The following review
    provides an overview of RNA-seq data analysis, including information about
    read count tables (Fig 2a and 2b are especially relevant):

    > Conesa, A., Madrigal, P., Tarazona, S. et al. A survey of best practices
    > for RNA-seq data analysis. Genome Biol 17, 13 (2016).
    > https://doi.org/10.1186/s13059-016-0881-8"""

    path: str


@Output(
    phase=2,
    phase_description="Downstream Analysis",
)
class DifferentialGeneExpression:
    """Test for differential gene expression

    The goal of this step is to assign a score (like a _p_-value) to each gene
    that ranks how differentially expressed it is between two conditions.
    Among other uses, this information can be plotted in an
    [MA plot](https://en.wikipedia.org/wiki/MA_plot) or a
    [volcano plot](https://en.wikipedia.org/wiki/Volcano_plot_(statistics%29).

    This analysis goal is a downstream analysis in RNA-seq processing.
    The following image shows a _typical_ RNA-seq processing workflow,
    **but the details can vary a lot!**

    ![An overview of the RNA-seq workflow.](assets/rna-seq.png)"""

    path: str


@Output(
    phase=2,
    phase_description="Downstream Analysis",
)
class PCAGeneExpression:
    """PCA - Perform 2D principal component analysis of gene expression

    This analysis computes a 2D visualization of gene expression using
    something called
    [principal component analysis](https://en.wikipedia.org/wiki/Principal_component_analysis).

    The "principal components" capture the axes of largest variation in the dataset.
    This means that if a quantity of interest segregates cleanly in the 2D
    PCA projection, you can have high assurance that it accounts for most of the
    variation in your dataset.

    For example, say you have samples in Condition A and samples in Condition
    B in your experiment, and you cluster each sample's gene expressions down
    to a 2D visualization using PCA. If all the Condition A points are close
    to each other and all the Condition B points are close to each other, that
    means that most of the variation in your data set can be explained by which
    condition your samples are assigned to.

    This analysis goal is a downstream analysis in RNA-seq processing.
    The following image shows a _typical_ RNA-seq processing workflow,
    **but the details can vary a lot!**

    ![An overview of the RNA-seq workflow.](assets/rna-seq.png)"""

    path: str


@Output(
    phase=2,
    phase_description="Downstream Analysis",
)
class CountDifferentiallyExpressedGenes:
    """Count differentially-expressed genes

    The goal of this step is to count how many genes are differentially
    expressed in each comparison.

    There are two thresholds you can set for this step:

    - The **p-value** threshold (significance threshold)
    - The **log2 fold change** threshold (magnitude threshold)

    The p-value tells you how significant your hit is (which is related to
    both sample size and magnitude) and the log2 fold change tells you just the
    magnitude of the differential expression.

    This analysis goal is a downstream analysis in RNA-seq processing.
    The following image shows a _typical_ RNA-seq processing workflow,
    **but the details can vary a lot!**

    ![An overview of the RNA-seq workflow.](assets/rna-seq.png)"""

    path: str


@Output(
    phase=2,
    phase_description="Downstream Analysis",
)
class AnyDifferentiallyExpressedGenes:
    """@intermediate:Filter genes to just differentially-expressed

    The goal of this step is to filter your entire list of genes to **just**
    those that are differentially expressed **in at least one of the
    comparisons against each control**.

    There are two thresholds you can set for this step:

    - The **p-value** threshold (significance threshold)
    - The **log2 fold change** threshold (magnitude threshold)

    The p-value tells you how significant your hit is (which is related to
    both sample size and magnitude) and the log2 fold change tells you just the
    magnitude of the differential expression."""

    path: str


@Output(
    phase=2,
    phase_description="Downstream Analysis",
)
class DifferentiallyExpressedGeneClusters:
    """Cluster genes differentially expressed in at least one comparison

    The goal of this step is to group genes that are differentially expressed
    among **any** of the comparisons against your control into **clusters**.

    Each cluster has genes that have similar expression patterns across your
    samples. For example, one cluster might contain genes that go up in your
    treatment condition, and another might contain genes that go down.

    Genes in the same cluster are often (but not always) involved in related
    biological processes, so these clusters are commonly used as input to
    downstream analyses like
    [gene ontology (GO)](https://en.wikipedia.org/wiki/Gene_Ontology)
    enrichment.

    This analysis goal is a downstream analysis in RNA-seq processing.
    The following image shows a _typical_ RNA-seq processing workflow,
    **but the details can vary a lot!**

    ![An overview of the RNA-seq workflow.](assets/rna-seq.png)"""

    path: str


# @Output(
#     phase=2,
#     phase_description="Downstream Analysis",
# )
# class RunBatchCorrection:
#     """@intermediate:Run batch correction?

#     If you have run your samples across conditions in a two or more "batches,"
#     then you may want to run _batch correction_, which corrects for systematic
#     differences between the batches.

#     For example, say you have samples from condition A and B. You run batch 1
#     of A and B on one day, and batch 2 on another day. Because you are
#     interested in comparing conditions A and B (and _not_ batches 1 and 2),
#     you may want to run batch correction to try to eliminate and systematic
#     differences between batches 1 and 2, leaving only the differences between
#     conditions A and B."""

#     path: str


# @Function(
#     search=False,
# )
# def run_batch_correction(__hb_ret: RunBatchCorrection):
#     """Run batch correction

#     This choice will enable batch correction in downstream analyses."""

#     # PARAMETER
#     RUN_BATCH_CORRECTION = True


# @Function(
#     search=False,
# )
# def do_not_run_batch_correction(__hb_ret: RunBatchCorrection):
#     """Do not run batch correction

#     This choice will skip batch correction."""

#     # PARAMETER
#     RUN_BATCH_CORRECTION = False


@Function(
    "ret.qc = false",
    "ret.trimmed = false",
    "ret.long = false",
    "ret.type = 'rna'",
    "P_RnaSeq {}",
    search=False,
)
def load_rna_seq(__hb_ret: SeqReads):
    """Load RNA-seq data from sample sheet

    This code collects all the RNA-seq data defined in the sample sheet. It
    downloads samples identified with SRR accession identifiers from the
    [European Nucleotide Archive](https://www.ebi.ac.uk/ena/browser/home)."""

    # PARAMETER: Path to RNA-seq sample sheet CSV
    RNA_SEQ_SAMPLE_SHEET = "sample_sheet.csv"

    sample_sheet = pl.read_csv(RNA_SEQ_SAMPLE_SHEET)

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
                assert file[-2:] in {"_1", "_2"}, f"SRR '{file}' must end in _1 or _2"

                # Construct the URL
                srr = file[:-2]  # remove _1 or _2
                base_url = "ftp://ftp.sra.ebi.ac.uk/vol1/fastq/"
                base_url += srr[:6] + "/"
                base_url += srr[9:].zfill(3) + "/"
                base_url += srr + "/"

                new_file = file + ".fastq.gz"

                # Download the file
                # -x 4 -s 4 means to use 4 concurrent streams to download the
                # files; please do not set this number too high out of respect
                # for the EBI's bandwidth!
                bash(f"""
                     aria2c
                         -x 4 -s 4
                         --dir={__hb_ret.path}
                         {base_url}{new_file}
                """)

            # Otherwise, symlink the file path (on the local machine)
            else:
                new_file = os.path.basename(file)
                link(
                    src=file,
                    dst=f"{__hb_ret.path}/{new_file}",
                )

            new_files[file] = new_file

    # Create the new sample sheet with updated filenames
    sample_sheet.with_columns(
        forward_location=pl.col("forward_location").replace(new_files),
        reverse_location=pl.col("reverse_location").replace(new_files),
    ).write_csv(f"{shared()}/sample_sheet.csv")

    # Create comparison sheet
    (
        sample_sheet.select("comparison", "condition")
        .unique(maintain_order=True)
        .with_columns(
            control_condition=pl.col("condition").first().over("comparison"),
        )
        .filter(pl.col("condition") != pl.col("control_condition"))
        .select("comparison", "control_condition", treatment_condition="condition")
        .write_csv(f"{shared()}/comparison_sheet.csv")
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
    use="to remove sequencing adapters AND poly(A)-tails from mRNA",
)
def cutadapt_rna(__hb_reads: SeqReads, __hb_ret: SeqReads):
    """cutadapt - remove poly(A) tails

    # Remove sequencing adapters and poly(A) tails using [cutadapt](https://cutadapt.readthedocs.io/en/stable/)

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
    >
    > Cleaning your data in this way is often required: Reads from small-RNA
    > sequencing contain the 3’ sequencing adapter because the read is longer
    > than the molecule that is sequenced. Amplicon reads start with a primer
    > sequence. Poly-A tails are useful for pulling out RNA from your sample,
    > but often you don’t want them to be in your reads.
    >
    > Cutadapt helps with these trimming tasks by finding the adapter or primer
    > sequences in an error-tolerant way."""

    # PARAMETER: The forward adapter, by default the Illumina universal
    FORWARD_ADAPTER = "AGATCGGAAGAGCACACGTCTGAACTCCAGTCA"

    # PARAMETER: The reverse adapter, by default the Illumina universal (can delete for single-end only)
    REVERSE_ADAPTER = "AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT"

    # PARAMETER: The number of cores to use
    CORES = 4

    # Load the sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Run cutadapt for each sample in the sample sheet
    for sample in sample_sheet.rows(named=True):
        # --cores number of cores, -m 1 remove reads <1, -a forward adapter
        # -A reverse adapter, -o forward output, -o reverse output
        # ==> Important! The --poly-a flag removes poly(A) tails!
        bash(f"""
            uv run cutadapt
                --cores={CORES}
                -m 1
                --poly-a
                -a {FORWARD_ADAPTER}
                -A {REVERSE_ADAPTER}
                -o {__hb_ret.path}/{sample["forward_location"]}
                -p {__hb_ret.path}/{sample["reverse_location"]}
                {__hb_reads.path}/{sample["forward_location"]}
                {__hb_reads.path}/{sample["reverse_location"]}
        """)


# # Single-end reads
# else:
#     # --cores number of cores, -m 1 remove reads <1, -a forward adapter
#     # -o forward output

#     # Important! The --poly-a flag removes poly(A) tails!

#     bash(f"""uv run cutadapt
#                 --cores={CORES}
#                 -m 1
#                 --poly-a
#                 -a {FORWARD_ADAPTER}
#                 -o {__hb_ret.path}/{sample["forward_location"]}
#                 {__hb_reads.path}/{sample["forward_location"]}""")


@Function(search=False)
def create_kallisto_index(__hb_ret: KallistoIndex):
    """Create necessary kallisto indexes by downloading Ensembl transcriptomes

    This code downloads transcriptomes from [Ensembl](https://ensembl.org) that match
    the sample sheet information, then creates a kallisto index for each of
    them. It also downloads the corresponding metadata for the transcriptome.

    A _transcriptome_ contains the complementary DNA (cDNA) for your transcripts
    of interest. Ensembl has such transcriptomes for many different species!"""

    # PARAMETER: The number of cores to use
    CORES = 4

    # PARAMETER: The version of Ensembl to use for gene annotations
    ENSEMBL_VERSION = "111"

    # Load the sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Get the species (e.g. homo_sapiens)
    species = sample_sheet["species"].unique().item()

    # Create transcriptome folder (if necessary)
    bash(f"""mkdir -p {shared()}/transcriptomes""")

    # Download the transcriptome from Ensembl
    bash(f"""
        wget
            --progress=bar:force
            -O {shared()}/transcriptomes/{species}.cdna.fa.gz
            $(uv run gget ref --ftp --which cdna --release {ENSEMBL_VERSION} {species})
    """)

    # Create the kallisto index
    # -t number of cores, -i output filename for index
    bash(f"""
        kallisto index
            -t {CORES}
            -i {__hb_ret.path}/kallisto.idx
            {shared()}/transcriptomes/{species}.cdna.fa.gz
    """)

    # Also download the Ensembl metadata
    bash(f"""mkdir -p {shared()}/{species}""")
    bash(f"""
        Rscript environment/fetch_ensembl_metadata.r
            --ensembl_version={ENSEMBL_VERSION}
            --species={species}
            --output_directory={shared()}/{species}
    """)


@Function(
    search=False,
)
def create_salmon_index(__hb_ret: SalmonIndex):
    """Create necessary Salmon indexes by downloading Ensembl transcriptomes

    This code downloads transcriptomes from [Ensembl](https://ensembl.org) that
    match the sample sheet information, then creates a Salmon index for each
    of them. It also downloads the corresponding metadata for the transcriptome.

    A _transcriptome_ contains the complementary DNA (cDNA) for your transcripts
    of interest. Ensembl has such transcriptomes for many different species!"""

    # PARAMETER: The number of cores to use
    CORES = 4

    # PARAMETER: The version of Ensembl to use for gene annotations
    ENSEMBL_VERSION = "111"

    # Load the sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Get the species (e.g. homo_sapiens)
    species = sample_sheet["species"].unique().item()

    # Create transcriptome folder (if necessary)
    bash(f"""mkdir -p {shared()}/transcriptomes""")

    # Download the transcriptome from Ensembl
    bash(f"""
        wget
            --progress=bar:force
            -O {shared()}/transcriptomes/{species}.cdna.fa.gz
            $(uv run gget ref --ftp --which cdna --release {ENSEMBL_VERSION} {species})
    """)

    # Create the Salmon index
    # -p number of cores, -t path to transcriptome, -i output filename for index
    bash(f"""
        salmon index
            -p {CORES}
            -t {shared()}/transcriptomes/{species}.cdna.fa.gz
            -i {__hb_ret.path}/salmon_index
    """)

    # Also download the Ensembl metadata
    bash(f"""mkdir -p {shared()}/{species}""")
    bash(f"""
        Rscript environment/fetch_ensembl_metadata.r
            --ensembl_version={ENSEMBL_VERSION}
            --species={species}
            --output_directory={shared()}/{species}
    """)


@Function(
    search=False,
)
def create_star_index(__hb_ret: StarIndex):
    """Create necessary STAR indexes by downloading Ensembl genomes

    This code downloads reference genomes and gene annotations from
    [Ensembl](https://ensembl.org) that match the sample sheet information,
    then creates a STAR index for each of them. It also downloads the
    corresponding metadata for the genome.

    Unlike a _transcriptome_ (which contains only the sequences of mRNA
    transcripts), a _genome_ contains the full DNA sequence of an organism.
    The gene annotation (in [GTF format](https://useast.ensembl.org/info/website/upload/gff.html))
    describes where each gene (and each exon of each gene) is located in the
    genome. Ensembl has such genomes and annotations for many different
    species!

    **Note:** Creating a STAR index for a large genome takes a lot of memory
    (about 32 GB of RAM for the human genome)."""

    # PARAMETER: The number of cores to use
    CORES = 4

    # PARAMETER: The version of Ensembl to use for gene annotations
    ENSEMBL_VERSION = "111"

    # Load the sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Get the species (e.g. homo_sapiens)
    species = sample_sheet["species"].unique().item()

    # Create genome and annotation folders (if necessary)
    bash(f"""mkdir -p {shared()}/genomes""")
    bash(f"""mkdir -p {shared()}/annotations""")

    # Download the genome from Ensembl
    bash(f"""
        wget
            --progress=bar:force
            -O {shared()}/genomes/{species}.dna.fa.gz
            $(uv run gget ref --ftp --which dna --release {ENSEMBL_VERSION} {species})
    """)

    # Download the gene annotation from Ensembl
    bash(f"""
        wget
            --progress=bar:force
            -O {shared()}/annotations/{species}.gtf.gz
            $(uv run gget ref --ftp --which gtf --release {ENSEMBL_VERSION} {species})
    """)

    # Decompress the genome and gene annotation (STAR requires uncompressed
    # files); -f overwrites existing files
    bash(f"""gunzip -f {shared()}/genomes/{species}.dna.fa.gz""")
    bash(f"""gunzip -f {shared()}/annotations/{species}.gtf.gz""")

    # Create the STAR index
    # --runThreadN number of cores, --genomeDir output folder for index,
    # --genomeFastaFiles path to genome, --sjdbGTFfile path to gene annotation,
    # --outFileNamePrefix where to put log files
    bash(f"""mkdir -p {__hb_ret.path}/star_index""")
    bash(f"""
        STAR
            --runMode genomeGenerate
            --runThreadN {CORES}
            --genomeDir {__hb_ret.path}/star_index
            --genomeFastaFiles {shared()}/genomes/{species}.dna.fa
            --sjdbGTFfile {shared()}/annotations/{species}.gtf
            --outFileNamePrefix {__hb_ret.path}/
    """)

    # Also download the Ensembl metadata
    bash(f"""mkdir -p {shared()}/{species}""")
    bash(f"""
        Rscript environment/fetch_ensembl_metadata.r
            --ensembl_version={ENSEMBL_VERSION}
            --species={species}
            --output_directory={shared()}/{species}
    """)


# @Function(
#     search=False,
# )
# def use_existing_kallisto_index(__hb_ret: KallistoIndex):
#     """Use existing index

#     If you already have a kallisto index for the transcriptome you'd like to
#     use, you can use this code to skip creating a new one."""

#     # PARAMETER: The location of the kallisto transcriptome index on your computer
#     KALLISTO_INDEX = "ensembl115.Homo_sapiens.GRCh38.cdna.all.kallisto.idx"

#     link(
#         KALLISTO_INDEX,
#         f"{__hb_ret.path}/kallisto.idx",
#     )


# @Function(
#     search=False,
# )
# def create_hg38_kallisto_index(__hb_ret: KallistoIndex):
#     """New from HUMAN transcriptome (hg38)

#     This code creates a kallisto index from the human transcriptome (hg38). It
#     automatically downloads the human transcriptome for you from
#     [Ensembl](https://www.ensembl.org/Homo_sapiens/Info/Index)."""

#     # PARAMETER: The number of cores to use
#     CORES = 4

#     # PARAMETER: The version of Ensembl to use for gene annotations
#     ENSEMBL_VERSION = "111"

#     bash(f"""
#         wget
#             --progress=bar:force
#             --no-clobber
#             --directory-prefix {shared()}/transcriptomes
#             https://ftp.ensembl.org/pub/release-{ENSEMBL_VERSION}/fasta/homo_sapiens/cdna/Homo_sapiens.GRCh38.cdna.all.fa.gz
#     """)

#     # -t number of cores, -i output filename for index
#     bash(f"""
#         kallisto index
#             -t {CORES}
#             -i {__hb_ret.path}/kallisto.idx
#             {shared()}/transcriptomes/Homo_sapiens.GRCh38.cdna.all.fa.gz
#     """)


# @Function(
#     search=False,
# )
# def create_kallisto_index(__hb_ret: KallistoIndex):
#     """New from OTHER transcriptome

#     This code creates a kallisto index from an existing transcriptome file on
#     your computer. This transcriptome file should contain the complementary
#     DNA (cDNA) for your transcripts of interest. A good place to download
#     transcriptome files is the
#     [Ensembl genome repository](https://www.ensembl.org/)."""

#     # PARAMETER: The number of cores to use
#     CORES = 4

#     # PARAMETER: The location of the reference transcriptome on your computer (FASTA format)
#     TRANSCRIPTOME_PATH = "ensembl115.Homo_sapiens.GRCh38.cdna.all.fa.gz"

#     # -t number of cores, -i output filename for index
#     bash(f"""
#         kallisto index
#             -t {CORES}
#             -i {__hb_ret.path}/kallisto.idx
#             {TRANSCRIPTOME_PATH}
#     """)


# @Function
# def use_existing_salmon_index(__hb_ret: SalmonIndex):
#     """Use existing index

#     If you already have a salmon index for the transcriptome you'd like to
#     use, you can use this code to skip creating a new one."""

#     # PARAMETER: The location of the salmon transcriptome index on your computer
#     SALMON_INDEX = "salmon_index"

#     link(
#         SALMON_INDEX,
#         f"{__hb_ret.path}/salmon_index",
#     )


# @Function(
#     search=False,
# )
# def create_hg38_salmon_index(__hb_ret: SalmonIndex):
#     """New from HUMAN transcriptome (hg38)

#     This code creates a salmon index from the human transcriptome (hg38). It
#     automatically downloads the human transcriptome for you from
#     [Ensembl](https://www.ensembl.org/Homo_sapiens/Info/Index)."""

#     # PARAMETER: The number of cores to use
#     CORES = 4

#     # PARAMETER: The version of Ensembl to use for gene annotations
#     ENSEMBL_VERSION = "111"

#     bash(f"""
#         wget
#             --progress=bar:force
#             --no-clobber
#             --directory-prefix {shared()}/transcriptomes
#             https://ftp.ensembl.org/pub/release-{ENSEMBL_VERSION}/fasta/homo_sapiens/cdna/Homo_sapiens.GRCh38.cdna.all.fa.gz
#     """)

#     # -p number of cores, -t path to transcriptome, -i output filename for index
#     bash(f"""
#         salmon index
#             -p {CORES}
#             -t {shared()}/transcriptomes/Homo_sapiens.GRCh38.cdna.all.fa.gz
#             -i {__hb_ret.path}/salmon_index
#     """)


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

    # Load sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Run kallisto for each sample in the sample sheet
    for sample in sample_sheet.rows(named=True):
        # -t number of cores, -i kallisto index, -o output folder
        bash(f"""
            kallisto quant
                -t {CORES}
                -i {__hb_idx.path}/kallisto.idx
                -o {__hb_ret.path}/{sample["sample_name"]}
                {__hb_reads.path}/{sample["forward_location"]}
                {__hb_reads.path}/{sample["reverse_location"]}
        """)


# # Single-end reads
# else:
#     # IMPORTANT: For single-end reads, you *must* set fragment length
#     # and standard deviation of the library! These can be determined
#     # from, e.g., a TapeStation. The kallisto creators note:
#     #     Typical Illumina libraries produce fragment lengths ranging
#     #     from 180–200 bp but it's best to determine this from a library
#     #     quantification with an instrument such as an Agilent
#     #     Bioanalyzer.
#     #                     - https://pachterlab.github.io/kallisto/manual
#     bash(f"""kallisto quant
#                 --fragment-length=200
#                 --sd=20
#                 -t {CORES}
#                 -i {__hb_idx.path}/kallisto.idx
#                 -o {__hb_ret.path}/{sample["sample_name"]}
#                 {__hb_reads.path}/{sample["forward_location"]}""")


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

    # Load sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Run kallisto for each sample in the sample sheet
    for sample in sample_sheet.rows(named=True):
        # -b number of bootstrap resamplings, -t number of cores,
        # -i kallisto index, -o output folder
        bash(f"""
             kallisto quant
                 -b {KALLISTO_BOOTSTRAPS}
                 -t {CORES}
                 -i {__hb_idx.path}/kallisto.idx
                 -o {__hb_ret.path}/{sample["sample_name"]}
                 {__hb_reads.path}/{sample["forward_location"]}
                 {__hb_reads.path}/{sample["reverse_location"]}
        """)


# # Single-end reads
# else:
#     # IMPORTANT: For single-end reads, you *must* set fragment length
#     # and standard deviation of the library! These can be determined
#     # from, e.g., a TapeStation. The kallisto creators note:
#     #     Typical Illumina libraries produce fragment lengths ranging
#     #     from 180–200 bp but it's best to determine this from a library
#     #     quantification with an instrument such as an Agilent
#     #     Bioanalyzer.
#     #                     - https://pachterlab.github.io/kallisto/manual
#     bash(f"""kallisto quant
#                 -b {KALLISTO_BOOTSTRAPS}
#                 --fragment-length=200
#                 --sd=20
#                 -t {CORES}
#                 -i {__hb_idx.path}/kallisto.idx
#                 -o {__hb_ret.path}/{sample["sample_name"]}
#                 {__hb_reads.path}/{sample["forward_location"]}""")


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

    # Run Salmon for each sample in the sample sheet
    for sample in sample_sheet.rows(named=True):
        # -p number of cores, -i salmon index, -1 forward reads, -2 reverse
        # reads, -o output folder (-l A autodetects library type)
        bash(f"""
            salmon quant
                -l A
                -p {CORES}
                -i {__hb_idx.path}/salmon_index
                -1 {__hb_reads.path}/{sample["forward_location"]}
                -2 {__hb_reads.path}/{sample["reverse_location"]}
                -o {__hb_ret.path}/{sample["sample_name"]}
        """)

        # Convert Salmon's quant.sf to kallisto's abundance.tsv format for
        # compatability
        pl.read_csv(
            f"{__hb_ret.path}/{sample['sample_name']}/quant.sf",
            separator="\t",
        ).select(
            # Rename left columns to right
            pl.col("Name").alias("target_id"),
            pl.col("Length").alias("length"),
            pl.col("EffectiveLength").alias("eff_length"),
            pl.col("NumReads").alias("est_counts"),
            pl.col("TPM").alias("tpm"),
        ).write_csv(
            # Save the file
            f"{__hb_ret.path}/{sample['sample_name']}/abundance.tsv",
            separator="\t",
        )


# # Single-end
# else:
#     # -p number of cores, -i salmon index, -1 forward reads, -2 reverse
#     # reads, -o output folder (-l A autodetects library type)
#     bash(f"""salmon quant
#                  -l A
#                 -p {CORES}
#                 -i {__hb_idx.path}/salmon_index
#                 -r {__hb_reads.path}/{sample["forward_location"]}
#                 -o {__hb_ret.path}/{sample["sample_name"]}""")

# # Single-end
# else:
#     # -p number of cores, -i salmon index, -1 forward reads, -2 reverse
#     # reads, -o output folder (-l A autodetects library type)
#     bash(f"""salmon quant
#                  -l A
#                 -p {CORES}
#                 -i {__hb_idx.path}/salmon_index
#                 -r {__hb_reads.path}/{sample["forward_location"]}
#                 -o {__hb_ret.path}/{sample["sample_name"]}""")


# # Single-end
# else:
#     # -p number of cores, -i salmon index, -1 forward reads, -2 reverse
#     # reads, -o output folder (-l A autodetects library type)
#     bash(f"""salmon quant
#                  -l A
#                 -p {CORES}
#                 -i {__hb_idx.path}/salmon_index
#                 -r {__hb_reads.path}/{sample["forward_location"]}
#                 -o {__hb_ret.path}/{sample["sample_name"]}""")


@Function(
    "reads.qc = true",
    "reads.trimmed = true",
    "reads.long = false",
    "reads.type = 'rna'",
    "ret.type = reads.type",
    "ret.compressed = true",
    pmid="23104886",
    citation="Dobin A, Davis CA, Schlesinger F, Drenkow J, Zaleski C, Jha S, "
    "Batut P, Chaisson M, Gingeras TR. STAR: ultrafast universal RNA-seq "
    "aligner. Bioinformatics. 2013 Jan 1;29(1):15-21. "
    "doi: 10.1093/bioinformatics/bts635. PMID: 23104886; PMCID: PMC3530905.",
    use="to align RNA-seq reads to a reference genome (splice-aware)",
)
def star(
    __hb_idx: StarIndex,
    __hb_reads: SeqReads,
    __hb_ret: SeqAlignment,
):
    """STAR

    # Align RNA-seq reads to a reference genome using [STAR](https://github.com/alexdobin/STAR)

    STAR is a _splice-aware_ aligner: it can align reads that span exon-exon
    junctions (where introns have been spliced out of the mRNA) to the
    genome. Alignment is much slower than transcript quantification methods
    like [kallisto](https://pachterlab.github.io/kallisto/) or
    [Salmon](https://salmon.readthedocs.io/en/latest/index.html), but it tells
    you exactly where each read aligns in the genome, which is useful for
    viewing the alignments (e.g. in [IGV](https://igv.org/)) or for discovering
    new splice junctions. To get gene-level read counts from the alignments,
    use a tool like
    [featureCounts](https://subread.sourceforge.net/featureCounts.html).

    This code outputs **sorted** and **indexed** BAM files.

    **Note:** Aligning to a large genome takes a lot of memory (about 32 GB of
    RAM for the human genome)!"""

    # PARAMETER: The number of cores to use
    CORES = 4

    # Load sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Run STAR for each sample in the sample sheet
    for sample in sample_sheet.rows(named=True):
        # Get the reads (works for paired-end and single-end)
        read_files = f"{__hb_reads.path}/{sample['forward_location']}"
        if sample["reverse_location"]:
            read_files += f" {__hb_reads.path}/{sample['reverse_location']}"

        # Decompress the reads on the fly if they are gzipped
        read_files_command = ""
        if sample["forward_location"].endswith(".gz"):
            read_files_command = "--readFilesCommand gunzip -c"

        # --runThreadN number of cores, --genomeDir STAR index, --readFilesIn
        # forward (and reverse) reads, --outFileNamePrefix prefix for output
        # files, --outSAMtype output a BAM file sorted by coordinate
        bash(f"""
            STAR
                --runThreadN {CORES}
                --genomeDir {__hb_idx.path}/star_index
                --readFilesIn {read_files}
                {read_files_command}
                --outFileNamePrefix {__hb_ret.path}/{sample["sample_name"]}.
                --outSAMtype BAM SortedByCoordinate
        """)

        # Rename the alignment to match the other aligners
        bash(f"""
            mv
                {__hb_ret.path}/{sample["sample_name"]}.Aligned.sortedByCoord.out.bam
                {__hb_ret.path}/{sample["sample_name"]}.bam
        """)

        # Index the alignment
        bash(f"""samtools index {__hb_ret.path}/{sample["sample_name"]}.bam""")


@Function(
    "data.bootstrapped = false",
    google_scholar_id="5898741618830664005",
    pmid="26925227",
    citation="Soneson C, Love MI, Robinson MD (2015). Differential analyses "
    "for RNA-seq: transcript-level estimates improve gene-level inferences. "
    "F1000Research, 4. doi:10.12688/f1000research.7563.1.",
)
def tximport(__hb_data: TranscriptMatrices, __hb_ret: GeneMatrices):
    """tximport (alignment-free method)

    # Aggregate transcript-level estimated counts for gene-level analysis with [tximport](https://bioconductor.org/packages/release/bioc/html/tximport.html)

    Tools like [kallisto](https://pachterlab.github.io/kallisto/) and
    [salmon](https://salmon.readthedocs.io/en/latest/) report transcript-level
    read counts, but many analyses of interest (such as differential _gene_
    expression) require gene-level data. tximport aggregates transcripts of the
    same gene together for gene-level downstream analysis.

    **Note:** This choice will force you to select an **alignment-free**
    quantification method like kallisto or Salmon!**"""

    # Load the sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Get the species (e.g. homo_sapiens)
    species = sample_sheet["species"].unique().item()

    # Run tximport
    bash(f"""
        Rscript environment/tximport.r
            --sample_sheet={shared()}/sample_sheet.csv
            --tx2gene={shared()}/{species}/tx2gene.csv
            --input_directory={__hb_data.path}
            --output_directory={__hb_ret.path}
    """)


@Function(
    "align.type = 'rna'",
    "align.compressed = true",
    pmid="24227677",
    citation="Liao Y, Smyth GK and Shi W (2014). featureCounts: an efficient "
    "general purpose program for assigning sequence reads to genomic features. "
    "Bioinformatics, 30(7):923-30.",
)
def featureCounts(__hb_align: SeqAlignment, __hb_ret: GeneMatrices):
    """featureCounts (alignment-based method)

    # Count reads aligned to each gene with [featureCounts](https://subread.sourceforge.net/featureCounts.html)

    Aligners like [STAR](https://github.com/alexdobin/STAR) report where each
    read aligns in the genome, but many analyses of interest (such as
    differential gene expression) require gene-level read counts.
    featureCounts uses a gene annotation (which describes where each gene is
    located in the genome) to tally up how many reads align to each gene.

    By default, reads that overlap more than one gene, or that align to more
    than one location in the genome, are **not** counted (because it's
    ambiguous which gene they came from).

    **Note:** This choice will force you to select an **alignment-based**
    quantification method like STAR!**"""

    # PARAMETER: The number of cores to use
    CORES = 4

    # Load the sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Get the species (e.g. homo_sapiens)
    species = sample_sheet["species"].unique().item()

    # Get the alignment for each sample
    alignments = {
        f"{__hb_align.path}/{sample_name}.bam": sample_name
        for sample_name in sample_sheet["sample_name"]
    }

    # Run featureCounts
    # -T number of cores, -a gene annotation, -o output file
    # -p --countReadPairs is for paired-end reads
    bash(f"""
        featureCounts
            -T {CORES}
            -p --countReadPairs
            -a {shared()}/annotations/{species}.gtf
            -o {__hb_ret.path}/featureCounts.tsv
            {" ".join(alignments)}
    """)

    # Load featureCounts output (rows are genes, columns are gene information
    # followed by the read counts for each alignment)
    results = pl.read_csv(
        f"{__hb_ret.path}/featureCounts.tsv",
        separator="\t",
        comment_prefix="#",
    )

    # Convert to the same format as tximport for compatibility (rows are
    # genes, columns are samples; unnamed first column the gene ID)
    results.select(
        pl.col("Geneid").alias(""),
        *[pl.col(path).alias(name) for path, name in alignments.items()],
    ).write_csv(f"{__hb_ret.path}/counts.csv")

    # Also compute transcripts-per-million (TPM) abundance: counts per base of
    # gene length (summed exon lengths), scaled to sum to one million
    results.select(
        pl.col("Geneid").alias(""),
        *[
            (
                (pl.col(path) / pl.col("Length"))
                / (pl.col(path) / pl.col("Length")).sum()
                * 1_000_000
            ).alias(name)
            for path, name in alignments.items()
        ],
    ).write_csv(f"{__hb_ret.path}/abundance.csv")


RUN_BATCH_CORRECTION = None

# --batch_correct={RUN_BATCH_CORRECTION}


@Function(
    use="a 2D visualization of your samples' gene expression",
    citation="Scikit-learn: Machine Learning in Python, Pedregosa et al., JMLR 12, pp. 2825-2830, 2011.",
)
def pca_gene_expression(
    __hb_data: GeneMatrices,
    # __hb_bc: RunBatchCorrection,
    __hb_ret: PCAGeneExpression,
):
    """2D Principal Component Analysis (PCA) with scikit-learn

    This function uses [scikit-learn](https://scikit-learn.org/stable/index.html)
    to perform a principal component analysis. Specifically, it uses the
    [`sklearn.decomposition.PCA`](https://scikit-learn.org/stable/modules/generated/
    sklearn.decomposition.PCA.html) function; that webpage contains all the
    documentation for this function and more examples."""

    # Load sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Load gene expression (rows are genes, columns are samples), dropping gene IDs
    expression = pl.read_csv(f"{__hb_data.path}/counts.csv").drop("")

    # Log-transform the counts so that highly-expressed genes don't dominate
    expression = expression.select((pl.all() + 1).log(base=2))

    # Transpose gene expression so that rows are samples, columns are genes
    X = expression.to_numpy().T

    # Get the principal components
    pcs = skdec.PCA(n_components=2).fit_transform(X)

    # Convert to dataframe and add condition information
    results = (
        pl.DataFrame(pcs, schema=["PC1", "PC2"])
        .insert_column(
            0,
            pl.Series("sample_name", expression.columns),
        )
        .join(sample_sheet, on="sample_name")
    )

    # Save the results of the PCA
    results.write_csv(
        f"{__hb_ret.path}/pca.tsv",
        separator="\t",
    )

    # Display the interactive PCA chart
    display(
        alt.Chart(results)
        .mark_point()
        .encode(
            x="PC1:Q",
            y="PC2:Q",
            color="condition:N",
            tooltip="sample_name:N",
        ),
    )


@Function(
    search=False,
)
def count_differentially_expressed_genes(
    __hb_data: DifferentialGeneExpression,
    __hb_ret: CountDifferentiallyExpressedGenes,
):
    """Count differentially-expressed genes

    This code counts how many genes are differentially expressed in at least one
    comparison against a control.

    You can tweak the parameters to set the **p-value** threshold (significance
    threshold) and **log2 fold change** threshold (magnitude threshold)."""

    # PARAMETER: The log2 fold change threshold for what constitutes a "differentially-expressed gene"
    RNA_SEQ_LOG2_FC_THRESHOLD = 1

    # PARAMETER: The p-value threshold for what constitutes a "differentially-expressed gene" after multiple comparisons correction
    RNA_SEQ_P_VALUE_THRESHOLD = 0.05

    # Load comparison sheet
    comparisons = pl.read_csv(f"{shared()}/comparison_sheet.csv")

    # Count the differentially-expressed genes for each comparison
    counts = []
    for comparison in comparisons.iter_rows(named=True):
        control = comparison["control_condition"]
        treatment = comparison["treatment_condition"]

        # Load the differential expression results
        results = pl.read_csv(
            os.path.join(
                __hb_data.path,
                control,
                treatment,
                "results.csv",
            ),
            null_values=["NA"],
        )

        # Keep only genes that pass both thresholds (null p-values are dropped)
        de_genes = results.filter(
            (pl.col("padj") < RNA_SEQ_P_VALUE_THRESHOLD)
            & (pl.col("log2FoldChange").abs() >= RNA_SEQ_LOG2_FC_THRESHOLD)
        )

        counts.append(
            {
                "comparison": comparison["comparison"],
                "control_condition": control,
                "treatment_condition": treatment,
                "differentially_expressed_genes": de_genes.height,
            }
        )

    # Aggregate the counts (one row per comparison)
    counts = pl.DataFrame(counts)

    # Save the counts
    counts.write_csv(f"{__hb_ret.path}/counts.csv")

    # Display the counts
    display(counts)


@Function(
    search=False,
)
def any_differentially_expressed_genes(
    __hb_data: DifferentialGeneExpression,
    __hb_ret: AnyDifferentiallyExpressedGenes,
):
    """List genes differentially expressed in any comparison

    This code filters your list of genes into just the genes that are
    differentially expressed in at least one comparison against a control.

    You can tweak the parameters to set the **p-value** threshold (significance
    threshold) and **log2 fold change** threshold (magnitude threshold)."""

    # PARAMETER: The log2 fold change threshold for what constitutes a "differentially-expressed gene"
    RNA_SEQ_LOG2_FC_THRESHOLD = 1

    # PARAMETER: The p-value threshold for what constitutes a "differentially-expressed gene" after multiple comparisons correction
    RNA_SEQ_P_VALUE_THRESHOLD = 0.05

    # Load the sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Get the species (e.g. homo_sapiens)
    species = sample_sheet["species"].unique().item()

    # Load gene metadata
    gene_metadata = pl.read_csv(f"{shared()}/{species}/gene_metadata.csv")

    # Carry over the gene read count tables (counts.csv and abundance.csv)
    carry_over(f"{__hb_data.path}/*.csv", __hb_ret.path)

    # Load comparison sheet
    comparisons = pl.read_csv(f"{shared()}/comparison_sheet.csv")

    # Collect the differentially-expressed genes for each comparison id
    for comparison_id, group in comparisons.group_by(
        "comparison",
        maintain_order=True,
    ):
        de_genes = []
        for comparison in group.iter_rows(named=True):
            control = comparison["control_condition"]
            treatment = comparison["treatment_condition"]

            # Load the differential expression results
            results = pl.read_csv(
                os.path.join(
                    __hb_data.path,
                    control,
                    treatment,
                    "results.csv",
                ),
                null_values=["NA"],
            ).rename({"": "gene_id"})

            # Keep only genes that pass both thresholds (null p-values are dropped)
            de_genes.append(
                results.filter(
                    (pl.col("padj") < RNA_SEQ_P_VALUE_THRESHOLD)
                    & (pl.col("log2FoldChange").abs() >= RNA_SEQ_LOG2_FC_THRESHOLD)
                ).select("gene_id")
            )

        # Combine genes across all treatments in this comparison id
        genes = (
            pl.concat(de_genes)
            .unique(maintain_order=True)
            .join(gene_metadata, on="gene_id", how="left")
        )

        # Save the genes
        genes.write_csv(
            f"{__hb_ret.path}/differentially_expressed_genes{comparison_id[0]}.csv"
        )


@Function(
    use="a clustering method that just splits genes into a prespecified number of clusters",
    citation="Scikit-learn: Machine Learning in Python, Pedregosa et al., JMLR 12, pp. 2825-2830, 2011.",
)
def kmeans_cluster_genes(
    __hb_genes: AnyDifferentiallyExpressedGenes,
    __hb_ret: DifferentiallyExpressedGeneClusters,
):
    """k-means clustering with scikit-learn

    This function uses [scikit-learn](https://scikit-learn.org/stable/index.html)
    to group genes into _k_ clusters using
    [k-means clustering](https://en.wikipedia.org/wiki/K-means_clustering).
    Specifically, it uses the
    [`sklearn.cluster.KMeans`](https://scikit-learn.org/stable/modules/generated/
    sklearn.cluster.KMeans.html) function; that webpage contains all the
    documentation for this function and more examples.

    Before clustering, each gene's expression is log-transformed and standardized
    (using [z-scores](https://en.wikipedia.org/wiki/Standard_score)) across
    samples, so that genes are clustered by the shape of their expression
    pattern rather than by how highly expressed they are overall."""

    # PARAMETER: The number of clusters to group genes into for the RNA-seq analysis
    RNA_SEQ_CLUSTER_COUNT = 4

    # Load gene expression (rows are genes, columns are samples), then
    # log-transform it and z-score each gene across samples
    expression = pl.read_csv(f"{__hb_genes.path}/abundance.csv").rename({"": "gene_id"})
    samples = [c for c in expression.columns if c != "gene_id"]
    expression = (
        expression.with_columns((pl.col(s) + 1).log(base=2) for s in samples)
        .with_columns(
            mean=pl.mean_horizontal(samples),
            std=pl.concat_list(samples).list.std(),
        )
        .filter(pl.col("std") > 0)
        .select(
            "gene_id",
            *[((pl.col(s) - pl.col("mean")) / pl.col("std")).alias(s) for s in samples],
        )
    )

    # Load comparison sheet
    comparisons = pl.read_csv(f"{shared()}/comparison_sheet.csv")

    # Cluster the differentially-expressed genes for each comparison id
    for comparison_id in comparisons["comparison"].unique(maintain_order=True):
        # Load the differentially-expressed genes and their expression
        genes = pl.read_csv(
            f"{__hb_genes.path}/differentially_expressed_genes{comparison_id}.csv"
        ).join(expression, on="gene_id", how="inner")

        # Run k-means clustering (rows are genes, columns are samples)
        X = genes.select(samples).to_numpy()
        labels = skclust.KMeans(
            n_clusters=min(RNA_SEQ_CLUSTER_COUNT, len(genes)),
            random_state=0,
        ).fit_predict(X)

        # Save the cluster assignments
        clusters = genes.select(pl.exclude(samples)).with_columns(
            cluster=pl.Series(labels + 1)
        )
        clusters.write_csv(f"{__hb_ret.path}/clusters{comparison_id}.csv")


@Function(
    use="a clustering method that builds a **tree** of similar genes.",
    citation="Scikit-learn: Machine Learning in Python, Pedregosa et al., JMLR 12, pp. 2825-2830, 2011.",
)
def hierarchical_cluster_genes(
    __hb_genes: AnyDifferentiallyExpressedGenes,
    __hb_ret: DifferentiallyExpressedGeneClusters,
):
    """Hierarchical clustering with scikit-learn

    This function uses [scikit-learn](https://scikit-learn.org/stable/index.html)
    to group genes into clusters using
    [hierarchical (agglomerative) clustering](https://en.wikipedia.org/wiki/Hierarchical_clustering),
    which repeatedly merges the most similar genes and groups of genes into a
    tree, then cuts the tree into a fixed number of clusters. Specifically, it
    uses the
    [`sklearn.cluster.AgglomerativeClustering`](https://scikit-learn.org/stable/modules/generated/
    sklearn.cluster.AgglomerativeClustering.html) function; that webpage
    contains all the documentation for this function and more examples.

    Before clustering, each gene's expression is log-transformed and standardized
    (using [z-scores](https://en.wikipedia.org/wiki/Standard_score)) across
    samples, so that genes are clustered by the shape of their expression
    pattern rather than by how highly expressed they are overall."""

    # PARAMETER: The number of clusters to group genes into for the RNA-seq analysis
    RNA_SEQ_CLUSTER_COUNT = 4

    # Load gene expression (rows are genes, columns are samples), then
    # log-transform it and z-score each gene across samples
    expression = pl.read_csv(f"{__hb_genes.path}/abundance.csv").rename({"": "gene_id"})
    samples = [c for c in expression.columns if c != "gene_id"]
    expression = (
        expression.with_columns((pl.col(s) + 1).log(base=2) for s in samples)
        .with_columns(
            mean=pl.mean_horizontal(samples),
            std=pl.concat_list(samples).list.std(),
        )
        .filter(pl.col("std") > 0)
        .select(
            "gene_id",
            *[((pl.col(s) - pl.col("mean")) / pl.col("std")).alias(s) for s in samples],
        )
    )

    # Load comparison sheet
    comparisons = pl.read_csv(f"{shared()}/comparison_sheet.csv")

    # Cluster the differentially-expressed genes for each comparison id
    for comparison_id in comparisons["comparison"].unique(maintain_order=True):
        # Load the differentially-expressed genes and their expression
        genes = pl.read_csv(
            f"{__hb_genes.path}/differentially_expressed_genes{comparison_id}.csv"
        ).join(expression, on="gene_id", how="inner")

        # Run hierarchical clustering (rows are genes, columns are samples)
        # You can change the linkage to use alternative methods.
        # Valid options are: "ward", "complete", "average", "single"
        X = genes.select(samples).to_numpy()
        labels = skclust.AgglomerativeClustering(
            n_clusters=min(RNA_SEQ_CLUSTER_COUNT, len(genes)),
            linkage="ward",
        ).fit_predict(X)

        # Save the cluster assignments
        clusters = genes.select(pl.exclude(samples)).with_columns(
            cluster=pl.Series(labels + 1)
        )
        clusters.write_csv(f"{__hb_ret.path}/clusters{comparison_id}.csv")


# <span class="more-info">
#     <span style="display: none">
#         A <b>negative binomial distribution</b> is a statistical distribution
#         that models count data with unexplained variance, such as the
#         number of mRNA transcripts in a cell.  It looks like an off-center
#         normal distribution that is only defined for non-negative integers.
#     </span>
#     negative binomial distribution</span>,


@Function(
    google_scholar_id="16121678637925818947",
    citation="Love MI, Huber W, Anders S (2014). Moderated estimation of fold change "
    "and dispersion for RNA-seq data with DESeq2. Genome Biology, 15, 550. "
    "doi:10.1186/s13059-014-0550-8.",
    pmid="25516281",
    use="a **widely-used tool** that does **not** give you error bars.",
)
def deseq2(
    __hb_data: GeneMatrices,
    # __hb_bc: RunBatchCorrection,
    __hb_ret: DifferentialGeneExpression,
):
    """DESeq2

    # Find differentially-expressed protein-coding genes with [DESeq2](https://bioconductor.org/packages/release/bioc/html/DESeq2.html)

    DESeq2 models gene expression using what is called a
    [negative binomial distribution](https://en.wikipedia.org/wiki/Negative_binomial_distribution),
    which is much more suitable for RNA-seq data than something like a _t_-test.
    DESeq2 has an
    [extensive guide](https://bioconductor.org/packages/release/bioc/vignettes/DESeq2/inst/doc/DESeq2.html)
    about how to use DESeq2 to analyze RNA-seq data.

    The above guide includes a very helpful
    [list of frequently-asked questions](https://bioconductor.org/packages/release/bioc/vignettes/DESeq2/inst/doc/DESeq2.html#frequently-asked-questions),
    including an explanation of why some adjusted _p_-values are `NA` and what
    can be done to turn off that behavior."""

    # PARAMETER: The p-value threshold for what constitutes a "differentially-expressed gene" after multiple comparisons correction
    RNA_SEQ_P_VALUE_THRESHOLD = 0.05

    # Load the sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Get the species (e.g. homo_sapiens)
    species = sample_sheet["species"].unique().item()

    # Run DESeq2
    bash(f"""
        Rscript environment/deseq2.r
            --gene_metadata={shared()}/{species}/gene_metadata.csv
            --sample_sheet={shared()}/sample_sheet.csv
            --comparison_sheet={shared()}/comparison_sheet.csv
            --gene_counts={__hb_data.path}/counts.csv
            --output_directory={__hb_ret.path}
    """)

    # Carry over the gene read count tables (counts.csv and abundance.csv)
    carry_over(f"{__hb_data.path}/*.csv", __hb_ret.path)

    # Load gene metadata
    gene_metadata = pl.read_csv(f"{shared()}/{species}/gene_metadata.csv")

    # Load comparison sheet
    comparisons = pl.read_csv(f"{shared()}/comparison_sheet.csv")

    # Create an interactive volcano plot for each comparison
    for comparison in comparisons.iter_rows(named=True):
        control = comparison["control_condition"]
        treatment = comparison["treatment_condition"]

        # Load the results from DESeq2 and give significance labels (up/down/ns)
        results = (
            pl.read_csv(
                os.path.join(
                    __hb_ret.path,
                    control,
                    treatment,
                    "results.csv",
                ),
                null_values=["NA"],
            )
            .rename({"": "gene_id"})
            .join(gene_metadata, on="gene_id")
            .with_columns(
                sig=pl.col("padj").log10().neg(),
                group=pl.when(
                    (pl.col("padj").is_null())
                    | (pl.col("padj") >= RNA_SEQ_P_VALUE_THRESHOLD)
                )
                .then(pl.lit("ns"))
                .when(pl.col("log2FoldChange") > 0)
                .then(pl.lit("up"))
                .otherwise(pl.lit("down")),
            )
        )

        # Make and display the volcano plot
        display(
            alt.Chart(results)
            .mark_point()
            .encode(
                x="log2FoldChange:Q",
                y="sig:Q",
                color=alt.Color(
                    "group:N",
                    scale=alt.Scale(
                        domain=["up", "down", "ns"],
                        range=["crimson", "steelblue", "black"],
                    ),
                ),
                tooltip=["gene_name", "gene_id"],
            )
            .properties(title=f"{treatment} vs. {control}"),
        )


@Function(
    "data.bootstrapped = true",
    google_scholar_id="1639708055766929241",
    pmid="28581496",
    citation="Harold J. Pimentel, Nicolas Bray, Suzette Puente, Páll Melsted "
    "and Lior Pachter, Differential analysis of RNA-Seq incorporating "
    "quantification uncertainty, Nature Methods (2017), "
    "http://dx.doi.org/10.1038/nmeth.4324.",
    use="a **lesser-used** (but still common) tool that **does** give you error bars.",
)
def sleuth(__hb_data: TranscriptMatrices, __hb_ret: DifferentialGeneExpression):
    """sleuth

    # Find differentially-expressed protein-coding genes with [sleuth](https://pachterlab.github.io/sleuth/)

    sleuth can find differentially-expressed genes between samples and can
    incorporate measurements of uncertainty using "bootstrap estimates" from
    read quantifiers like [kallisto](http://pachterlab.github.io/kallisto).
    sleuth also has a collection of [walkthroughs](https://pachterlab.github.io/sleuth/walkthroughs)
    that demonstrate how to use it to analyze RNA-seq datasets."""

    # PARAMETER: The number of cores to use
    CORES = 4

    # PARAMETER: The p-value threshold for what constitutes a "differentially-expressed gene" after multiple comparisons correction
    RNA_SEQ_P_VALUE_THRESHOLD = 0.05

    # Load the sample sheet
    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    # Get the species (e.g. homo_sapiens)
    species = sample_sheet["species"].unique().item()

    # Run sleuth
    bash(f"""
        Rscript environment/sleuth.r
            --cores={CORES}
            --gene_metadata={shared()}/{species}/gene_metadata.csv
            --tx2gene={shared()}/{species}/tx2gene.csv
            --sample_sheet={shared()}/sample_sheet.csv
            --comparison_sheet={shared()}/comparison_sheet.csv
            --input_directory={__hb_data.path}
            --output_directory={__hb_ret.path}
    """)

    # Also aggregate transcripts to gene read count tables (counts.csv and
    # abundance.csv) for downstream analyses
    bash(f"""
        Rscript environment/tximport.r
            --sample_sheet={shared()}/sample_sheet.csv
            --tx2gene={shared()}/{species}/tx2gene.csv
            --input_directory={__hb_data.path}
            --output_directory={__hb_ret.path}
    """)

    # Load gene metadata
    gene_metadata = pl.read_csv(f"{shared()}/{species}/gene_metadata.csv")

    # Load comparison sheet
    comparisons = pl.read_csv(f"{shared()}/comparison_sheet.csv")

    # Create an interactive volcano plot for each comparison
    for comparison in comparisons.iter_rows(named=True):
        control = comparison["control_condition"]
        treatment = comparison["treatment_condition"]

        # Load the results from sleuth and give significance labels (up/down/ns)
        results = (
            pl.read_csv(
                os.path.join(
                    __hb_ret.path,
                    control,
                    treatment,
                    "results.csv",
                ),
                null_values=["NA"],
            )
            .rename({"": "gene_id"})
            .join(gene_metadata, on="gene_id")
            .with_columns(
                sig=pl.col("padj").log10().neg(),
                group=pl.when(
                    (pl.col("padj").is_null())
                    | (pl.col("padj") >= RNA_SEQ_P_VALUE_THRESHOLD)
                )
                .then(pl.lit("ns"))
                .when(pl.col("log2FoldChange") > 0)
                .then(pl.lit("up"))
                .otherwise(pl.lit("down")),
            )
        )

        # Make and display the volcano plot
        display(
            alt.Chart(results)
            .mark_point()
            .encode(
                x="log2FoldChange:Q",
                y="sig:Q",
                color=alt.Color(
                    "group:N",
                    scale=alt.Scale(
                        domain=["up", "down", "ns"],
                        range=["crimson", "steelblue", "black"],
                    ),
                ),
                tooltip=["gene_name", "gene_id"],
            )
            .properties(title=f"{treatment} vs. {control}"),
        )


################################################################################
# %% LEMONmethyl-seq

# fastq_path: str
# """Path to the directory containing the LEMONmethyl-seq data

# @example:raw-data/

# This directory should contain files ending with `.fastq` or `.fastq.gz`."""

# reference: str
# """Path to the reference genome to align the LEMONmethyl-seq data to

# @example:reference.fasta

# The path should be to a `.fasta` file containing one entry (for the
# reference genome)."""


@Input
class LemonSeq:
    """LEMONmethyl-seq

    [Locus-Enriched Mapping Of Nucleotide methylation
    (LEMONmethyl-seq)](https://www.biorxiv.org/content/10.64898/2026.02.24.70776
    1v1) is a cost-effective pipeline for single-nucleotide detection of
    DNA methylation at target regions using locus-specific amplification and
    long-read DNA sequencing."""


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
    "P_LemonSeq {}",
    search=False,
)
def load_local_lemon_seq(__hb_ret: UnconvertedLemonSeq):
    """Load LEMONmethyl-seq data from hard drive

    The raw LEMONmethyl-seq files are typically in the `.fastq` or `.fastq.gz`
    file format."""

    # PARAMETER: Path to the directory containing the LEMONmethyl-seq data
    LEMON_FASTQ_PATH = "raw-data/"

    # PARAMETER: Path to the reference genome to align the LEMONmethyl-seq data to
    LEMON_REFERENCE = "reference.fasta"

    carry_over(LEMON_FASTQ_PATH, __hb_ret.path)

    link(
        LEMON_REFERENCE,
        f"{shared()}/reference/unconverted.fasta",
    )

    log("Start of unconverted reference:\n")

    bash(f"""
        head "{shared()}/reference/unconverted.fasta"
    """)


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

    bash(f"""
        cat "{shared()}/reference/unconverted.fasta" \
            | sed '/^>/s/$/ (in silico C -> T converted)/' \
            | sed '/^[^>]/s/C/T/g' \
            | sed '/^[^>]/s/c/t/g' \
            > "{shared()}/reference/reference.fasta"
    """)

    log("Start of converted reference:\n")

    bash(f"""
        head "{shared()}/reference/reference.fasta"
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
    EM_REFERENCE_PATH = "genomes/converted.fasta"

    carry_over(__hb_data.path, __hb_ret.path)

    link(
        EM_REFERENCE_PATH,
        f"{shared()}/reference/reference.fasta",
    )

    log("Start of previously-converted reference:\n")

    bash(f"""
        head "{shared()}/reference/reference.fasta"
    """)


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

    for path in sorted(glob.glob(f"{__hb_bam.path}/*.bam")):
        sample_name = os.path.splitext(os.path.basename(path))[0]
        bash(f"""
            uv run environment/LEMONmC.py \
                --ref "{shared()}/reference/unconverted.fasta" \
                --bam "{path}"
                --tsv "{__hb_ret.path}/{sample_name}.tsv"
        """)


################################################################################
# %% EM-seq

# path: str
# """Path to the directory containing the raw EM-seq data

# @example:raw-data/

# This directory should contain files ending with `.fastq` or `.fastq.gz`."""


@Input
class LocalEmSeq:
    """EM-seq

    [Enzymatic Methyl-seq (EM-seq)](https://www.neb.com/en-us/products/e7120-nebnext-enzymatic-methyl-seq-kit)
    is a technique to detect genome-wide DNA methylation at single-nucleotide
    resolution that uses enzymes (rather than harsh bisulfite treatment) to
    convert unmethylated cytosines, preserving DNA integrity."""


@Output
class EmSeqNoRef:
    "@intermediate:EM-seq data (without converted reference)"

    path: str
    """Data path

    Directory structure:
    - *.fastq: raw EM-seq data
    """


@Function(
    "P_LocalEmSeq {}",
    search=False,
)
def load_local_em_seq(__hb_ret: EmSeqNoRef):
    """Load EM-seq data from hard drive

    # Load raw EM-seq data already present on your computer

    The raw EM-seq files are typically in the `.fastq` or `.fastq.gz` file
    format."""

    # PARAMETER: Path to the directory containing the raw EM-seq data
    EM_FASTQ_PATH = "raw-data/"

    carry_over(EM_FASTQ_PATH, __hb_ret.path)


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
    REFERENCE_GENOME_FOLDER = "genomes/genome_folder"

    bash(f"""
        bismark_genome_preparation \
            --verbose \
            --parallel 1 \
            {REFERENCE_GENOME_FOLDER}
    """)

    carry_over(__hb_input.path, __hb_ret.path)

    link(
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
    BISMARK_GENOME_FOLDER = "genomes/bismark_genome_folder"

    carry_over(__hb_input.path, __hb_ret.path)

    link(
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

    for path in sorted(glob.glob(f"{__hb_reads.path}/*{FORWARD_READ_SUFFIX}.fastq.gz")):
        sample_name = stem(path).removesuffix(FORWARD_READ_SUFFIX)
        log(f"Running bismark on '{path}'...")
        bash(f"""
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

    for path in sorted(glob.glob(f"{__hb_input.path}/*.bam")):
        log(f"Running bismark_methylation_extractor on '{path}'...")
        # Tip: Add --CX to the command below to get CpH methylation! (It takes
        # a lot more time and space to analyze, though.)
        bash(f"""
            bismark_methylation_extractor
               --parallel 1
               --gzip
               --bedGraph
               -o {__hb_ret.path}
               {path}
        """)


# # PARAMETER: The suffix at the end of the filenames for the forward reads
# FORWARD_READ_SUFFIX = "_R1"

# # Original top (OT) strand
# for path in glob.glob(f"{__hb_ret.path}/CpG_OT_*.txt.gz"):
#     sample_name = stem(path).removesuffix(FORWARD_READ_SUFFIX + "_bismark_bt2_pe")
#     log(f"Running bismark2bedGraph on '{path}' OT...")
#     bash(f"""
#         bismark2bedGraph \
#            --dir {__hb_ret.path} \
#            --parallel {max(BISMARK_CORES // 3, 1)} \
#            -o {sample_name}_OT.txt \
#            {path}
#     """)

# # Original bottom (OB) strand
# for path in glob.glob(f"{__hb_ret.path}/CpG_OB_*.txt.gz"):
#     sample_name = stem(path).removesuffix(FORWARD_READ_SUFFIX + "_bismark_bt2_pe")
#     log(f"Running bismark2bedGraph on '{path}' OB...")
#     bash(f"""
#         bismark2bedGraph \
#            --dir {__hb_ret.path} \
#            --parallel {max(BISMARK_CORES // 3, 1)} \
#            -o {sample_name}_OB.txt \
#            {path}
#     """)


################################################################################
# %% ATAC-seq analysis

# Tutorial: https://nbis-workshop-epigenomics.readthedocs.io/en/latest/content/tutorials/ATACseq/lab-atacseq-bulk.html#

# sample_sheet: str
# """Path to sample sheet CSV

# @example:metadata/sample_sheet.csv

# Here is an example CSV file (the headers must match exactly):

# | sample_name | condition | forward_location        | reverse_location        |
# |-------------|-----------|-------------------------|-------------------------|
# | BM001_t1    | treated   | raw-data/t1_R1.fastq.gz | raw-data/t1_R2.fastq.gz |
# | BM002_t2    | treated   | raw-data/t2_R1.fastq.gz | raw-data/t1_R2.fastq.gz |
# | BM003_u1    | untreated | SRR34323943_1           | SRR3423943_2            |
# | BM004_u2    | untreated | SRR34323942_1           | SRR3423942_2            |

# Each row is one sample. Here is what each column means:
# - **`sample_name`** is a unique identifier for each sample (it can be
#   whatever you want as long as it is unique).
# - **`condition`** is the label for the experimental condition for each
#   sample (this label can be whatever you want, such as "control" and
#   "treatment"). Multiple samples with the same condition are considered
#   **biological replicates**.
# - **`forward_location`** is the path to the raw ATAC-seq data of forward
#   reads, likely ending in `.fastq` or `.fastq.gz`. For paired-end reads,
#   the filename is likely to end in something resembling `_1.fastq.gz` or
#   `_R1.fastq.gz`. Optionally, you can refer to pre-existing datasets using
#   their SRA "run accession" (SRR) identifier. These identifiers are of the
#   form SRR*xxxxxxxx*, where each *x* is digit. For both single-end and
#   paired-end data, add a _1 to the end of the SRR identifier.
# - **`reverse_location`** (for **paired-end data only**) is the path to the
#   reverse reads. This path should be similar to the `forward_location` path
#   but have `_2` or `_R2` in the filename instead of `_1` or `_R1`. To refer
#   to a pre-existing experiment with an SRR, add `_2` to the end of the SRR
#   identifier."""


@Input
class AtacSeq:
    """ATAC-seq

    [Assay for Transposase-Accessible Chromatin using sequencing (ATAC-seq)](https://www.nature.com/articles/nmeth.2688)
    is a technique to identify regions of open (accessible) chromatin across
    the genome, which often correspond to active regulatory elements like
    promoters and enhancers."""


@Output
class AtacPeaks:
    """Chromatin accessibility peaks"""

    path: str


@Function(
    "ret.qc = false",
    "ret.trimmed = false",
    "ret.long = false",
    "ret.type = 'atac'",
    "P_AtacSeq {}",
    search=False,
)
def load_atac_seq(__hb_ret: SeqReads):
    """Load ATAC-seq data from sample sheet

    This code collects all the ATAC-seq data defined in the sample sheet. It
    downloads samples identified with SRR accession identifiers from the
    [European Nucleotide Archive](https://www.ebi.ac.uk/ena/browser/home)."""

    # PARAMETER: Path to RNA-seq sample sheet CSV
    ATAC_SEQ_SAMPLE_SHEET = "metadata/sample_sheet.csv"

    sample_sheet = pl.read_csv(ATAC_SEQ_SAMPLE_SHEET)

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
                assert file[-2:] in {"_1", "_2"}, f"SRR '{file}' must end in _1 or _2"

                # Construct the URL
                srr = file[:-2]  # remove _1 or _2
                base_url = "ftp://ftp.sra.ebi.ac.uk/vol1/fastq/"
                base_url += srr[:6] + "/"
                base_url += srr[9:].zfill(3) + "/"
                base_url += srr + "/"

                new_file = file + ".fastq.gz"

                # Download the file
                # -x 16 -s 16 means to use 16 concurrent streams to download the
                # files; please do not set this number too high out of respect
                # for the EBI's bandwidth!
                bash(f"""
                     aria2c
                         -x 16 -s 16
                         --dir={__hb_ret.path}
                         {base_url}{new_file}
                """)

            # Otherwise, symlink the file path (on the local machine)
            else:
                new_file = os.path.basename(file)
                link(
                    src=file,
                    dst=f"{__hb_ret.path}/{new_file}",
                )
            new_files[file] = new_file

    # Create the new sample sheet with updated filenames
    sample_sheet.with_columns(
        forward_location=pl.col("forward_location").replace(new_files),
        reverse_location=pl.col("reverse_location").replace(new_files),
    ).write_csv(f"{shared()}/sample_sheet.csv")


@Function(
    "align.compressed = true",
    "align.type = 'atac'",
)
def macs3(__hb_align: SeqAlignment, __hb_ret: AtacPeaks):
    """MACS3

    # Call chromatin accessibility peaks with [MACS3](https://macs3-project.github.io/MACS/index.html)

    "Peaks" of chromatin accessibility are regions that are regions of DNA that
    are significantly enriched in a sample compared to the background. Detecting
    these regions is useful for DNA enrichment assays such as ATAC-seq and
    ChIP-seq."""

    # PARAMETER: Effective genome size (hs for human, mm for mouse, or a number; see https://deeptools.readthedocs.io/en/develop/content/feature/effectiveGenomeSize.html)
    GENOME_SIZE = "hs"

    sample_sheet = pl.read_csv(f"{shared()}/sample_sheet.csv")

    for sample in sample_sheet.rows(named=True):
        # Paired-end
        if sample["reverse_location"]:
            input_format = "BAMPE"
        # Single-end
        else:
            input_format = "BAM"

        # -t input (treatment), -f format, -g genome size, -n output name,
        # --outdir output directory
        bash(f"""
             uv run macs3 callpeak
                 -t {__hb_align.path}/{sample["sample_name"]}.bam
                 -f {input_format}
                 -g {GENOME_SIZE}
                 -n {sample["sample_name"]}
                 --outdir {__hb_ret.path}
        """)


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
