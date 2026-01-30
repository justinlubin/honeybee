################################################################################
# %% Imports

library(tidyverse)
library(biomaRt)
library(sleuth)

################################################################################
# %% Command-line arguments

args = commandArgs(trailingOnly = TRUE)

ENSEMBL_VERSION = args[1]
ENSEMBL_DATASET = args[2]
SAMPLE_SHEET = args[3]
COMPARISON_SHEET = args[4]
DATA_DIR = args[5]
OUTPUT_DIR = args[6]

################################################################################
# %% Main script

# Get gene metadata (transcript-level for sleuth)

mart = useEnsembl(
    biomart = "ensembl",
    dataset = ENSEMBL_DATASET,
    version = ENSEMBL_VERSION
)

gene_metadata = getBM(
    attributes = c(
        "ensembl_transcript_id",
        "ensembl_gene_id",
        "external_gene_name"
    ),
    mart = mart
)

gene_metadata <- dplyr::rename(
    gene_metadata,
    target_id = ensembl_transcript_id,
    ens_gene = ensembl_gene_id,
    ext_gene = external_gene_name
)

write_csv(gene_metadata, file.path(OUTPUT_DIR, "gene_metadata.csv"))

# Load sample sheet and comparisons sheet

metadata = read.csv(
    SAMPLE_SHEET,
    header = TRUE,
)

s2c = data.frame(
    sample = metadata$sample_name,
    condition = metadata$condition,
    path = file.path(DATA_DIR, metadata$sample_name)
)

comparisons = read.csv(
    COMPARISON_SHEET,
    header = TRUE,
)

# Run comparisons

for (i in 1:nrow(comparisons)) {
    row = comparisons[i,]

    filtered_s2c <- s2c %>% filter(
        condition %in% c(row$control, row$treatment)
    )

    so <- sleuth_prep(
        filtered_s2c,
        target_mapping = gene_metadata,
        extra_bootstrap_summary = FALSE,
        read_bootstrap_tpm = FALSE,
        num_cores = 1,
        ~condition
    )

    so <- sleuth_fit(so, ~condition, 'full')
    so <- sleuth_fit(so, ~1, 'reduced')
    so <- sleuth_lrt(so, 'reduced', 'full')

    sleuth_table <- sleuth_results(so, 'reduced:full', 'lrt')

    filename = paste0(
        paste(row$control, row$treatment, sep = "-"),
        ".csv"
    )

    write_csv(sleuth_table, file.path(OUTPUT_DIR, filename))
}
