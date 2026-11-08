################################################################################
# %% Imports

library(optparse)
library(sleuth)

################################################################################
# %% Command-line arguments

parser = OptionParser()

parser = add_option(
  parser,
  c("--cores"),
  type = "integer",
  help = "Number of cores to use"
)

parser = add_option(
  parser,
  c("--gene_metadata"),
  type = "character",
  help = "Path to gene metadata file"
)

parser = add_option(
  parser,
  c("--tx2gene"),
  type = "character",
  help = "Path to transcript-to-gene file"
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
  c("--input_directory"),
  type = "character",
  help = "Path to input directory (kallisto quantifications with bootstraps)"
)

parser = add_option(
  parser,
  c("--output_directory"),
  type = "character",
  help = "Path to output directory"
)

opt = parse_args(parser)

CORES = opt$cores
GENE_METADATA = opt$gene_metadata
TX2GENE = opt$tx2gene
SAMPLE_SHEET = opt$sample_sheet
COMPARISON_SHEET = opt$comparison_sheet
INPUT_DIR = opt$input_directory
OUTPUT_DIR = opt$output_directory

################################################################################
# %% Main script

print("Loading gene metadata file...")

gene_metadata = read.csv(
  GENE_METADATA,
  header = TRUE
)

print("Loading tx2gene (transcript-to-gene) file...")

tx2gene = read.csv(
  TX2GENE,
  header = TRUE
)

print("Filtering to protein-coding genes...")

protein_coding = gene_metadata[
  gene_metadata$gene_biotype == "protein_coding",
  "gene_id"
]

target_mapping = data.frame(
  target_id = tx2gene$tx_id,
  gene_id = tx2gene$gene_id
)

target_mapping = target_mapping[
  target_mapping$gene_id %in% protein_coding,
]

print("Loading sample sheet...")

samples = read.csv(
  SAMPLE_SHEET,
  header = TRUE
)

print("Loading comparison sheet...")

comparisons = read.csv(
  COMPARISON_SHEET,
  header = TRUE
)

# Run comparisons

for (i in 1:nrow(comparisons)) {
  row = comparisons[i, ]

  print(paste(
    "Running sleuth with control '",
    row$control_condition,
    "' and treatment '",
    row$treatment_condition,
    "'",
    sep = ""
  ))

  # Set up sleuth input data

  control = samples[
    samples$condition == row$control_condition,
  ]

  treatment = samples[
    samples$condition == row$treatment_condition,
  ]

  sample_names = c(control$sample_name, treatment$sample_name)
  conditions = c(control$condition, treatment$condition)

  s2c = data.frame(
    sample = sample_names,
    condition = factor(
      conditions,
      levels = c(row$control_condition, row$treatment_condition)
    ),
    path = file.path(INPUT_DIR, sample_names)
  )

  # Run sleuth (aggregating transcripts to genes)

  so = sleuth_prep(
    s2c,
    full_model = ~condition,
    target_mapping = target_mapping,
    aggregation_column = "gene_id",
    gene_mode = TRUE,
    extra_bootstrap_summary = FALSE,
    num_cores = CORES
  )

  so = sleuth_fit(so, ~condition, "full")
  so = sleuth_fit(so, ~1, "reduced")

  # Likelihood ratio test for significance
  so = sleuth_lrt(so, "reduced", "full")

  # Wald test for the fold change
  beta = paste0("condition", row$treatment_condition)
  so = sleuth_wt(so, beta, "full")

  lrt = sleuth_results(so, "reduced:full", "lrt", show_all = FALSE)
  wt = sleuth_results(so, beta, "wt", show_all = FALSE)

  # Match the DESeq2 results format (gene IDs as row names). In gene mode,
  # target_id is the gene ID. sleuth's b (and its standard error) is a fold
  # change on the natural log scale, so convert it to log2. The p-values come
  # from the likelihood ratio test, so stat is the likelihood ratio test
  # statistic.
  wt = wt[match(lrt$target_id, wt$target_id), ]

  res = data.frame(
    log2FoldChange = wt$b / log(2),
    lfcSE = wt$se_b / log(2),
    stat = lrt$test_stat,
    pvalue = lrt$pval,
    padj = lrt$qval,
    row.names = lrt$target_id
  )

  # Write results to CSV

  print("Writing results...")

  path = file.path(
    OUTPUT_DIR,
    row$control_condition,
    row$treatment_condition,
    "results.csv"
  )

  dir.create(dirname(path), recursive = TRUE, showWarnings = FALSE)
  write.csv(res, path)
}
