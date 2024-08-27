import os.path
import subprocess

MEMOIZE = True


def should_run(path):
    if MEMOIZE and os.path.exists(path):
        print(f"(Skipping {path})")
        return False
    return True


# %%

print("Run setup")

DATA = "intermediates/Jarret20"
subprocess.run(
    f"""mkdir -p {DATA}""",
    shell=True,
)

# %%

print("Download SRA ascension numbers")

if should_run(f"{DATA}/SRR.txt"):
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

with open(f"{DATA}/SRR.txt") as f:
    srrs = [line.strip() for line in f.readlines()]

srrs

# %%

for srr in srrs:
    print(f"Download SRR file: {srr}")
    if should_run(f"{DATA}/{srr}/{srr}.sra"):
        subprocess.run(
            f"""prefetch {srr} -p --output-directory {DATA}""",
            shell=True,
        )

# %%

for srr in srrs:
    print(f"Convert SRR file to paired-end FASTQ file: {srr}")
    if should_run(f"{DATA}/{srr}/{srr}_1.fastq"):
        subprocess.run(
            f"""fasterq-dump {DATA}/{srr}/{srr}.sra --outdir {DATA}/{srr}""",
            shell=True,
        )

# %%

for srr in srrs:
    print(f"Run FastQC for {srr}_1")
    if should_run(f"{DATA}/{srr}/{srr}_1_fastqc.html"):
        subprocess.run(
            f"""fastqc {DATA}/{srr}/{srr}_1.fastq --outdir {DATA}/{srr}""",
            shell=True,
        )

    print(f"Run FastQC for {srr}_2")
    if should_run(f"{DATA}/{srr}/{srr}_2_fastqc.html"):
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
    if should_run(f"{DATA}/{srr}/trimmed.1.fastq"):
        subprocess.run(
            f"""
                cutadapt \
                  --cores=0 \
                  --poly-a \
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

if should_run(f"{DATA}/mm10.zip"):
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
