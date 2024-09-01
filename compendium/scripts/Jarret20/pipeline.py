# %%

import os.path
import subprocess

# %%

print("Running setup")

DATA = "data/Jarret20"
subprocess.run(
    f"""mkdir -p {DATA}""",
    shell=True,
)

# %%

print("Downloading SRA runs")

subprocess.run(
    f"""
        esearch -db sra -query PRJNA594861 \
          | efetch -format runinfo \
          | cut -d "," -f 1 \
          | tail -n +2 \
          > {DATA}/SRR.txt
    """,
    shell=True,
)

# %%

with open(f"{DATA}/SRR.txt") as f:
    srrs = [line.strip() for line in f.readlines()]

srrs

# %%

for srr in srrs:
    print(f"Downloading SRR file: {srr}")
    subprocess.run(
        f"""prefetch {srr} -p --output-directory {DATA}""",
        shell=True,
    )

# %%

for srr in srrs:
    print(f"Converting SRR file to paired-end FASTQ file: {srr}")
    subprocess.run(
        f"""fasterq-dump {DATA}/{srr}/{srr}.sra --outdir {DATA}/{srr}""",
        shell=True,
    )

# %%

for srr in srrs:
    print(f"Running FastQC for {srr}_1")
    subprocess.run(
        f"""fastqc {DATA}/{srr}/{srr}_1.fastq --outdir {DATA}/{srr}""",
        shell=True,
    )

    print(f"Runnning FastQC for {srr}_2")
    subprocess.run(
        f"""fastqc {DATA}/{srr}/{srr}_2.fastq --outdir {DATA}/{srr}""",
        shell=True,
    )

# https://support-docs.illumina.com/SHARE/AdapterSequences/Content/SHARE/AdapterSeq/TruSeq/SingleIndexes.htm
# TrueSeq Indexed adapter: (A)GATCGGAAGAGCACACGTCTGAACTCCAGTCAC-N*
# TruSeq Universal adapter: AATGATACGGCGACCACCGAGATCTACACTCTTTCCCTACACGACGCTCTTCCGATCT
# Illumina recommends following seqs:
# Read 1/fwd: AGATCGGAAGAGCACACGTCTGAACTCCAGTCA
# Read 2/rev: AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT (rev complement of universal ending)
# See also: http://tucf-genomics.tufts.edu/documents/protocols/TUCF_Understanding_Illumina_TruSeq_Adapters.pdf

# %%

for srr in srrs:
    print("Trimming read {srr}")
    # Discard empty reads for STAR
    subprocess.run(
        f"""
            cutadapt \
              --cores=0 \
              --poly-a \
              -m 1 \
              -a AGATCGGAAGAGCACACGTCTGAACTCCAGTCA \
              -A AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT \
              -o {DATA}/{srr}/trimmed.1.fastq \
              -p {DATA}/{srr}/trimmed.2.fastq \
              {DATA}/{srr}/{srr}_1.fastq \
              {DATA}/{srr}/{srr}_2.fastq
        """,
        shell=True,
    )

# %%

print("Downloading mm10")

subprocess.run(
    f"""
        datasets download genome accession \
            GCF_000001635.26 \
            --include genome,gtf \
            --filename {DATA}/mm10.zip \
        && unzip {DATA}/mm10.zip \
        && mv {DATA}/ncbi_dataset/ {DATA}/mm10
    """,
    shell=True,
)

# %%

print("STAR indexing")

subprocess.run(
    f"""
        mkdir -p {DATA}/star_genome && \
        STAR \
            --runThreadN 32 \
            --runMode genomeGenerate \
            --genomeDir {DATA}/star_genome \
            --genomeFastaFiles {DATA}/mm10/data/GCF_000001635.26/GCF_000001635.26_GRCm38.p6_genomic.fna \
            --sjdbGTFfile {DATA}/mm10/data/GCF_000001635.26/genomic.gtf
    """,
    shell=True,
)

# %%

for srr in srrs:
    print(f"STAR mapping {srr}")
    subprocess.run(
        f"""
            STAR \
                --runThreadN 32 \
                --genomeDir {DATA}/star_genome \
                --readFilesIn {DATA}/{srr}/trimmed.1.fastq {DATA}/{srr}/trimmed.2.fastq \
                --outFileNamePrefix {DATA}/{srr}/
        """,
        shell=True,
    )
