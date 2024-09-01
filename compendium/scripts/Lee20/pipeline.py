# %%

import os.path
import subprocess
import pandas as pd
import numpy as np

# %%

print("Running setup")

DATA = "data/Lee20"
subprocess.run(
    f"""mkdir -p {DATA}""",
    shell=True,
)

# %%

print("Downloading SRA runs")

subprocess.run(
    f"""
        echo "srr\ttitle" > {DATA}/runs.tsv && \
        esearch -db sra -query PRJNA548932 \
          | efetch -format xml \
          | xtract -pattern EXPERIMENT_PACKAGE \
              -element RUN@accession \
              -element SAMPLE/TITLE \
          >> {DATA}/runs.tsv
    """,
    shell=True,
)

# %%

print("Loading SRA runs")

runs = pd.read_csv(f"{DATA}/runs.tsv", sep="\t")
idx = (
    runs["title"].str.contains("TGF-b") | runs["title"].str.contains("rmSAA1")
) & runs["title"].str.contains("48hrs")
runs = runs[idx]

# %%

for srr in runs["srr"]:
    print(f"Downloading SRR file: {srr}")
    subprocess.run(
        f"""prefetch {srr} -p --output-directory {DATA}""",
        shell=True,
    )

# %%

for srr in runs["srr"]:
    print(f"Converting SRR file to paired-end FASTQ file: {srr}")
    subprocess.run(
        f"""fasterq-dump {DATA}/{srr}/{srr}.sra --outdir {DATA}/{srr}""",
        shell=True,
    )

# %%

for srr in runs["srr"]:
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


# %%

print("Downloading GRCm38 (mouse)")

subprocess.run(
    f"""
        datasets download genome accession \
            GCF_000001635.26 \
            --include genome,gtf \
            --filename {DATA}/GRCm38.zip \
        && unzip {DATA}/GRCm38.zip -d {DATA}/genome \
        && mv {DATA}/genome/ncbi_dataset/ {DATA}/GRCm38 \
    """,
    shell=True,
)
