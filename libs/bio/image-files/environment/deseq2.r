################################################################################
# %% Imports

# suppressPackageStartupMessages(library(tidyverse))
library(optparse)
library(DESeq2)

################################################################################
# %% Command-line arguments

parser = OptionParser()

parser = add_option(
  parser,
  c("--gene_metadata"),
  type = "character",
  help = "Path to gene metadata file"
)

parser = add_option(
  parser,
  c("--sample_sheet"),
  type = "character",
  help = "Path to sample sheet"
)

parser = add_option(
  parser,
  c("--gene_counts"),
  type = "character",
  help = "Path to gene read count matrix"
)

parser = add_option(
  parser,
  c("--output_directory"),
  type = "character",
  help = "Path to output directory"
)

opt = parse_args(parser)

GENE_METADATA = opt$gene_metadata
SAMPLE_SHEET = opt$sample_sheet
GENE_COUNTS = opt$gene_counts
OUTPUT_DIR = opt$output

################################################################################
# %% Main script

print("Loading gene metadata file...")

gene_metadata = read.csv(
  GENE_METADATA,
  header = TRUE
)

print("Filtering to protein-coding genes...")

protein_coding = gene_metadata[
  gene_metadata$gene_biotype == "protein_coding",
  "gene_id"
]

print("Loading sample sheet...")

samples = read.csv(
  SAMPLE_SHEET,
  header = TRUE,
)

print("Loading counts...")

counts = round(
  read.csv(
    GENE_COUNTS,
    row.names = 1
  )
)

print("Filtering genes to protein-coding only...")

counts = counts[
  rownames(counts) %in% protein_coding,
]

# Run comparisons

# TODO: make comparisons table

for (i in 1:nrow(comparisons)) {
  row = comparisons[i, ]

  paste(
    "Running DESeq2 with control '",
    row$control_condition,
    "' and treatment '",
    row$treatment_condition,
    "'",
    sep = ""
  )

  # Set up DESeq2 input data

  control = samples[
    samples$condition == row$control_condition,
  ]

  treatment = samples[
    samples$condition == row$treatment_condition,
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
    design = ~condition
  )

  dds = DESeq(dds)

  res = results(
    dds,
    contrast = c(
      "condition",
      row$treatment_condition,
      row$control_condition
    ),
  )

  # Write results to CSV

  paste("Writing results...")

  path = file.path(
    OUTPUT_DIR,
    row$control_condition,
    row$treatment_condition,
    "results.csv"
  )

  dir.create(path, recursive = TRUE, showWarnings = FALSE)
  write.csv(res, path)
}
