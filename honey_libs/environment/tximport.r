# TODO: Only works for kallisto

################################################################################
# %% Imports

library(biomaRt)
library(tximport)

################################################################################
# %% Command-line arguments

args = commandArgs(trailingOnly = TRUE)
ENSEMBL_VERSION = args[1]
ENSEMBL_DATASET = args[2]
SAMPLE_SHEET = args[3]
TRANSCRIPT_DIR = args[4]
OUTPUT_DIR = args[5]

################################################################################
# %% Main script

# Set up file paths

metadata = read.csv(SAMPLE_SHEET, header = TRUE)

files = file.path(
    TRANSCRIPT_DIR,
    metadata$sample,
    "abundance.h5"
)

names(files) = metadata$sample

# %% Create tx2gene using biomaRt

mart = useEnsembl(
    biomart = "ensembl",
    dataset = ENSEMBL_DATASET,
    version = ENSEMBL_VERSION
)

tx2gene = getBM(
    attributes =
        c("ensembl_transcript_id_version",
          "ensembl_gene_id_version"),
    mart = mart
)

# %% Aggregate abundances and save count file

txi = tximport(
    files,
    type = "kallisto",
    tx2gene = tx2gene
)

write.csv(txi$counts, file.path(OUTPUT_DIR, "counts.csv"))
write.csv(txi$abundance, file.path(OUTPUT_DIR, "abundance.csv"))
