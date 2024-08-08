import subprocess as s

# %% Setup

DATA = "intermediates/Jarret20"

# %% Download SRA ascension numbers

s.run(
    f"""
esearch -db sra -query PRJNA594861 \
  | efetch -format runinfo \
  | cut -d "," -f 1 \
  | tail -n +2 \
  > {DATA}/SRR.txt
""",
    shell=True,
)

# %% Download SRR files

with open(f"{DATA}/SRR.txt") as f:
    for line in f.readlines():
        srr = line.strip()
        s.run(
            f"""prefetch {srr} -p --output-directory {DATA}""",
            shell=True,
        )
