################################################################################
# %% Imports

suppressPackageStartupMessages(library(tidyverse))
suppressPackageStartupMessages(library(biomaRt))
suppressPackageStartupMessages(library(sleuth))
suppressPackageStartupMessages(library(optparse))

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

CORES = opt$cores
ENSEMBL_VERSION = opt$ensembl_version
ENSEMBL_DATASET = opt$ensembl_dataset
SAMPLE_SHEET = opt$sample_sheet
COMPARISON_SHEET = opt$comparison_sheet
DATA_DIR = opt$input
OUTPUT_DIR = opt$output

################################################################################
# %% Main script

# Get transcript metadata

transcript_metadata_filename = "output/shared/transcript_metadata.csv"

if (file.exists(transcript_metadata_filename)) {
    print("Transcript metadata already exists; loading it now...")

    transcript_metadata = read.csv(transcript_metadata_filename, header=TRUE)
} else {
    print("Transcript metadata does not exist yet; fetching it now...")
    
    print("Connecting to Ensembl...")

    mart = useEnsembl(
        biomart = "ensembl",
        dataset = ENSEMBL_DATASET,
        version = ENSEMBL_VERSION,
        host    = "may2025.archive.ensembl.org"
    )
    
    transcript_metadata = getBM(
        attributes = c(
            "ensembl_transcript_id_version",
            "ensembl_gene_id_version",
            "external_gene_name",
            "gene_biotype"
        ),
        mart = mart
    )

    write_csv(transcript_metadata, transcript_metadata_filename)
}

transcript_metadata = dplyr::rename(
    transcript_metadata,
    target_id = ensembl_transcript_id_version,
    ens_gene = ensembl_gene_id_version,
    ext_gene = external_gene_name
)

print("Filtering to protein-coding genes...")

transcript_metadata = transcript_metadata[
    transcript_metadata$gene_biotype == "protein_coding", 
]

# Load sample sheet and comparisons sheet

metadata = read.csv(
    SAMPLE_SHEET,
    header = TRUE,
)

# Convert to sleuth format
sample2condition = data.frame(
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

    filtered_s2c <- sample2condition %>% filter(
        condition %in% c(row$control, row$treatment)
    )

    so <- sleuth_prep(
        filtered_s2c,
        target_mapping = transcript_metadata,
        extra_bootstrap_summary = FALSE,
        read_bootstrap_tpm = FALSE,
        num_cores = CORES,
        ~condition
    )

    so <- sleuth_fit(so, ~condition, "full")
    so <- sleuth_fit(so, ~1, "reduced")
    so <- sleuth_lrt(so, "reduced", "full")

    sleuth_table <- sleuth_results(so, "reduced:full", "lrt")

    filename = paste0(
        paste(row$control, row$treatment, sep = "-"),
        ".csv"
    )

    write_csv(sleuth_table, file.path(OUTPUT_DIR, filename))
}
