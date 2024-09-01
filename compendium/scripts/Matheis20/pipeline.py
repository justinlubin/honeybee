# %%

import os.path
import subprocess
import pandas as pd
import numpy as np

# %%

print("Running setup")

DATA = "data/Matheis20"
subprocess.run(
    f"""mkdir -p {DATA}""",
    shell=True,
)

# %%

print("Downloading SRA runs")

subprocess.run(
    f"""
        echo "srr\ttitle" > {DATA}/runs.tsv && \
        esearch -db sra -query PRJNA589300 \
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
idx = runs["title"].str.startswith("Nodose") | runs["title"].str.startswith("Ileum")
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

print("Downloading GRCm39 (mouse); should be Mouse Ensembl v91")

subprocess.run(
    f"""
        datasets download genome accession \
            GCF_000001635.27 \
            --include genome,gtf \
            --filename {DATA}/GRCm39.zip \
        && unzip {DATA}/GRCm39.zip \
        && mv {DATA}/ncbi_dataset/ {DATA}/GRCm39
    """,
    shell=True,
)

# %%

print("Creating kallisto index")

subprocess.run(
    f"""
        kallisto index \
            -i {DATA}/kallisto.idx \
            {DATA}/GRCm39/data/GCF_000001635.27/GCF_000001635.27_GRCm39_genomic.fna
    """,
    shell=True,
)

# %%

for _, (srr, title) in runs.iterrows():
    print(f"Kalliso quantifying {srr}")

    # The paper says they follow the following protocol:
    # https://www.takarabio.com/documents/User%20Manual/SMARTer%20Ultra%20Low%20RNA%20Kit%20for%20Illumina%20Sequencing%20User%20Manual%20%28PT5163/SMARTer%20Ultra%20Low%20RNA%20Kit%20for%20Illumina%20Sequencing%20User%20Manual%20%28PT5163-1%29_010616.pdf
    # Page 17 says that they use the Covaris to shear to 200-500. Let's assume
    # a mean fragment length of (200+500)/2 = 350, with a standard deviation of
    # 100.
    subprocess.run(
        f"""
            kallisto quant \
                -i {DATA}/kallisto.idx \
                -o {DATA}/{srr} \
                -b 100 \
                --single \
                -l 350 \
                -s 100 \
                {DATA}/{srr}/{srr}.fastq
        """,
        shell=True,
    )
