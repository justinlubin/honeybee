################################################################################
# %% Imports

library(biomaRt)
library(tximport)
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
    c("--input"),
    type = "character",
    help = "Path to input directory (transcript quantifications)"
)

parser = add_option(
    parser,
    c("--output"),
    type = "character",
    help = "Path to output directory (gene quantifications)"
)

opt = parse_args(parser)

ENSEMBL_VERSION = opt$ensembl_version
ENSEMBL_DATASET = opt$ensembl_dataset
SAMPLE_SHEET = opt$sample_sheet
TRANSCRIPT_DIR = opt$input
OUTPUT_DIR = opt$output

###############################################################################
# %% Main script

# Set up file paths

print("Loading sample sheet...")

metadata = read.csv(SAMPLE_SHEET, header = TRUE)

files = file.path(
    TRANSCRIPT_DIR,
    metadata$sample_name,
    "abundance.tsv"
)

names(files) = metadata$sample_name

# %% Create tx2gene using biomaRt

print("Connecting to Ensembl...")

mart = useEnsembl(
    biomart = "ensembl",
    dataset = ENSEMBL_DATASET,
    #version = ENSEMBL_VERSION,
    mirror  = "www"
)

print("Fetching transcript-to-gene table...")

tx2gene = getBM(
    attributes =
        c("ensembl_transcript_id_version",
          "ensembl_gene_id_version"),
    mart = mart
)

print("Writing transcript-to-gene table...")

write.csv(tx2gene, "output/shared/tx2gene.csv")

# %% Aggregate abundances and save count file

print("Aggregating transcripts to genes...")

txi = tximport(
    files,
    type = "kallisto",
    tx2gene = tx2gene
)

print("Writing final abundance tables...")

write.csv(txi$counts, file.path(OUTPUT_DIR, "counts.csv"))
write.csv(txi$abundance, file.path(OUTPUT_DIR, "abundance.csv"))
