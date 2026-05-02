################################################################################
# %% Imports

library(tidyverse)
library(biomaRt)
library(DESeq2)

################################################################################
# %% Command-line arguments

args = commandArgs(trailingOnly = TRUE)

ENSEMBL_VERSION = args[1]
ENSEMBL_DATASET = args[2]
SAMPLE_SHEET = args[3]
COMPARISON_SHEET = args[4]
GENE_COUNTS = args[5]
OUTPUT_DIR = args[6]

################################################################################
# %% Main script

# Get gene metadata

mart = useEnsembl(
    biomart = "ensembl",
    dataset = ENSEMBL_DATASET,
    version = ENSEMBL_VERSION
)

gene_metadata = getBM(
    attributes = c(
          "ensembl_gene_id",
          "ensembl_gene_id_version",
          "chromosome_name",
          "start_position",
          "end_position",
          "external_gene_name",
          "gene_biotype"
    ),
    mart = mart
)

write_csv(gene_metadata, file.path(OUTPUT_DIR, "gene_metadata.csv"))

protein_coding = gene_metadata[
    gene_metadata$gene_biotype == "protein_coding",
    "ensembl_gene_id_version"
]

# Load sample sheet, comparisons sheet, and RNA-seq data

metadata = read.csv(
    SAMPLE_SHEET,
    header=TRUE,
)

comparisons = read.csv(
    COMPARISON_SHEET,
    header=TRUE,
)

counts = round(
    read.csv(
        GENE_COUNTS,
        row.names=1
    )
)

counts = counts[
    rownames(counts) %in% protein_coding,
]

# Run comparisons

for (i in 1:nrow(comparisons)) {
    # Set up DESeq2 input data

    row = comparisons[i,]
    control = metadata[
        (metadata$condition == row$control),
    ]
    treatment = metadata[
        (metadata$condition == row$treatment),
    ]

    cols = c(control$sample, treatment$sample)
    conditions = c(control$condition, treatment$condition)

    colData = data.frame(
        id = cols,
        condition = as.factor(conditions)
    )

    # Run DESeq2

    dds = DESeqDataSetFromMatrix(
        countData = counts[, cols],
        colData = colData,
        design = ~ condition
    )

    dds = DESeq(dds)

    res = results(
        dds,
        contrast=c("condition", row$treatment, row$control),
    )

    # Write results to CSV

    filename = paste0(
        paste(row$control, row$treatment, sep="-"),
        ".csv"
    )

    write.csv(
        res,
        file.path(OUTPUT_DIR, filename),
    )
}
