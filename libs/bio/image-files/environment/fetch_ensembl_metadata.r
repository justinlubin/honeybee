# Imports

library(optparse)
library(AnnotationHub)
library(ensembldb)

# Option parsing

parser = OptionParser()

parser = add_option(
  parser,
  c("--ensembl_version"),
  type = "character",
  help = "Ensembl version (e.g. 116)"
)

parser = add_option(
  parser,
  c("--species"),
  type = "character",
  help = "Species (e.g. homo_sapiens)"
)

parser = add_option(
  parser,
  c("--output_directory"),
  type = "character",
  help = "Path to output directory"
)

opt = parse_args(parser)

ENSEMBL_VERSION = opt$ensembl_version
SPECIES_WITH_UNDERSCORE = opt$species
OUTPUT_DIRECTORY = opt$output_directory

# Main script

# Replace _ with space for AnnotationHub
species = gsub("_", " ", SPECIES_WITH_UNDERSCORE)

print("Retrieving Ensembl database...")

ah = AnnotationHub()
q = query(ah, c("EnsDb", species, ENSEMBL_VERSION))
edb = q[[1]]

print("Fetching transcript-to-gene table from Ensembl...")

tx2gene = transcripts(
  edb,
  columns = c("tx_id", "gene_id"),
  return.type = "data.frame"
)

print("Writing transcript-to-gene table...")

write.csv(tx2gene, file.path(OUTPUT_DIRECTORY, "tx2gene.csv"))

print("Fetching gene metadata from Ensembl...")

gene_metadata = genes(
  edb,
  columns = c("gene_id", "gene_biotype"),
  return.type = "data.frame"
)

print("Saving gene metadata...")

write_csv(gene_metadata, file.path(OUTPUT_DIRECTORY, "gene_metadata.csv"))
