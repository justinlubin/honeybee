################################################################################
# %% Imports

library(tidyverse)
library(biomaRt)
library(DESeq2)
library(optparse)

################################################################################
# %% Command-line arguments

parser = OptionParser()

parser = add_option(
    parser,
    c("--ensembl_version"),
    type = "character",
    help = "Ensembl version to use (e.g. 115)"
)

parser = add_option(
    parser,
    c("--ensembl_dataset"),
    type = "character",
    help = "Ensembl dataset to use (e.g. hsapiens_gene_ensembl)"
)

parser = add_option(
    parser,
    c("--sample_sheet"),
    type = "character",
    help = "Path to sample sheet"
)

parser = add_option(
    parser,
    c("--comparison_sheet"),
    type = "character",
    help = "Path to comparison sheet"
)

parser = add_option(
    parser,
    c("--input"),
    type = "character",
    help = "Path to input directory (gene read count matrix)"
)

parser = add_option(
    parser,
    c("--output"),
    type = "character",
    help = "Path to output directory"
)

opt = parse_args(parser)

ENSEMBL_VERSION = opt$ensembl_version
ENSEMBL_DATASET = opt$ensembl_dataset
SAMPLE_SHEET = opt$sample_sheet
COMPARISON_SHEET = opt$comparison_sheet
GENE_COUNTS = opt$input
OUTPUT_DIR = opt$output

################################################################################
# %% Main script

# Get gene metadata

print("Connecting to Ensembl...")

mart = useEnsembl(
    biomart = "ensembl",
    dataset = ENSEMBL_DATASET,
    version = ENSEMBL_VERSION
)

print("Fetching gene metadata...")

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

print("Saving gene metadata...")

write_csv(gene_metadata, file.path(OUTPUT_DIR, "gene_metadata.csv"))

print("Filtering to protein-coding genes...")

protein_coding = gene_metadata[
    gene_metadata$gene_biotype == "protein_coding",
    "ensembl_gene_id_version"
]

# Load sample sheet, comparisons sheet, and RNA-seq data

print("Loading sample sheet...")

metadata = read.csv(
    SAMPLE_SHEET,
    header=TRUE,
)

print("Loading comparison sheet...")

comparisons = read.csv(
    COMPARISON_SHEET,
    header=TRUE,
)

print("Loading counts...")

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
    paste(
        "Running DESeq2 with control '",
        row$control_condition,
        "' and treatment '",
        row$treatment_condition,
        "'",
        sep=""
    )

    # Set up DESeq2 input data

    row = comparisons[i,]
    control = metadata[
        (metadata$condition == row$control_condition),
    ]
    treatment = metadata[
        (metadata$condition == row$treatment_condition),
    ]

    cols = c(control$sample_name, treatment$sample_name)
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
        contrast=c(
            "condition",
            row$treatment_condition,
            row$control_condition
        ),
    )

    # Write results to CSV

    paste("Writing results...")

    filename = paste0(
        paste(row$control_condition, row$treatment_condition, sep="-"),
        ".csv"
    )

    write.csv(
        res,
        file.path(OUTPUT_DIR, filename),
    )
}
