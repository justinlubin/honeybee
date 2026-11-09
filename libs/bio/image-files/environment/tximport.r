################################################################################
# %% Imports

library(optparse)
library(tximport)

################################################################################
# %% Command-line arguments

parser = OptionParser()

parser = add_option(
  parser,
  c("--sample_sheet"),
  type = "character",
  help = "Path to sample sheet"
)

parser = add_option(
  parser,
  c("--tx2gene"),
  type = "character",
  help = "Path to transcript-to-gene file"
)

parser = add_option(
  parser,
  c("--input_directory"),
  type = "character",
  help = "Path to input directory (transcript quantifications)"
)

parser = add_option(
  parser,
  c("--output_directory"),
  type = "character",
  help = "Path to output directory (gene quantifications)"
)

opt = parse_args(parser)

SAMPLE_SHEET = opt$sample_sheet
TX2GENE = opt$tx2gene
INPUT_DIR = opt$input_directory
OUTPUT_DIR = opt$output_directory

###############################################################################
# %% Main script

print("Loading sample sheet...")

metadata = read.csv(
  SAMPLE_SHEET,
  header = TRUE
)

files = file.path(
  INPUT_DIR,
  metadata$sample_name,
  "abundance.tsv"
)

names(files) = metadata$sample_name

print("Loading tx2gene (transcript-to-gene) file...")

tx2gene = read.csv(
  TX2GENE,
  header = TRUE
)

print("Aggregating transcripts to genes...")

txi = tximport(
  files,
  type = "kallisto",
  tx2gene = tx2gene
)

print("Writing final abundance tables...")

write.csv(
  txi$counts,
  file.path(OUTPUT_DIR, "counts.csv"),
)

write.csv(
  txi$abundance,
  file.path(OUTPUT_DIR, "abundance.csv"),
)
