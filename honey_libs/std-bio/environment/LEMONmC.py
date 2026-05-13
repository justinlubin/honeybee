import argparse
import sys

import pysam

###############################################################################
# Information

PROG = "LEMONmC"
VERSION = PROG + " 0.1.0 with pysam " + pysam.__version__
CITATION = f"""If you use {PROG} in a publication, please cite the LEMON-seq
paper using the following citation:

TODO

Please also cite samtools and HTSlib, which pysam relies on:

Li, H., Handsaker, B., Wysoker, A., Fennell, T., Ruan, J., Homer, N., Marth, G., Abecasis, G., Durbin, R., & 1000 Genome Project Data Processing Subgroup (2009). The Sequence Alignment/Map format and SAMtools. Bioinformatics (Oxford, England), 25(16), 2078–2079. https://doi.org/10.1093/bioinformatics/btp352

Bonfield, J. K., Marshall, J., Danecek, P., Li, H., Ohan, V., Whitwham, A., Keane, T., & Davies, R. M. (2021). HTSlib: C library for reading/writing high-throughput sequencing data. GigaScience, 10(2), giab007. https://doi.org/10.1093/gigascience/giab007"""

###############################################################################
# Helpers


def error(text: str) -> None:
    """Prints an error to the console and immediately quits."""

    print(text, file=sys.stderr)
    sys.exit(1)


def read_single_entry_fasta(fasta_path: str) -> tuple[str, str]:
    """Reads a FASTA file that contains a single entry and returns a pair of the
    sequence description and the sequence."""

    sequence = ""

    with open(fasta_path, "r") as f:
        first_line: str = next(f)

        # First line must be sequence description
        if not first_line.startswith(">"):
            error("error: reference fasta must start with >")

        # Parse description (drop > at start and strip whitespace)
        description = first_line[1:].strip()

        for line in f:
            # Remove whitespace (including trailing newline) and convert to
            # uppercase
            line = line.strip().upper()

            # If blank, that means the entry is over
            if not line:
                break

            # Append the sequence to return value
            sequence += line

        # Once the first entry is over, ensure that all remaining lines in the
        # file are blank (we enforce that input FASTA has only one entry)
        for line in f:
            if line.strip():
                error("error: reference fasta has more than one entry")

    return description, sequence


def cytosines(seq: str) -> dict[int, str]:
    """Returns the positions cytosines in a sequence and whether each is
    followed by a guanine (CpG), non-guanine (CpH), or unknown (Cp?) because
    the cytosine is at the end of the sequence. The keys of the returned
    dictionary are the cytosine positions, and the values are whether it is
    Cp?/CpG/CpH."""

    length = len(seq)
    ret = {}

    # Loop through each base in the sequence
    for pos, base in enumerate(seq):
        # Skip if non-cytosine
        if base != "C":
            continue

        if pos == length - 1:  # i.e., at final position
            kind = "Cp?"
        else:
            if seq[pos + 1] == "G":  # i.e., followed by G
                kind = "CpG"
            else:  # i.e., followed by non-G
                kind = "CpH"

        ret[pos] = kind

    return ret


###############################################################################
# Top-level main function


def call_methylation(bam_path: str, reference_path: str, tsv_path: str) -> None:
    """Call methylation from BAM files of EM-converted samples."""

    # Find all cytosines
    cs = cytosines(read_single_entry_fasta(reference_path)[1])

    # Open BAM file for reading ('r' = read, 'b' = BAM)
    with pysam.AlignmentFile(bam_path, "rb") as bam:
        # Open TSV file for writing
        with open(tsv_path, "w") as f:
            # Write header
            f.write("Pos\tMethylated\tUnmethylated\tPercent_Methylation\tC_Type\n")

            # Loop through each column in the alignment
            for col in bam.pileup():
                # This is a 0-based coordinate; we print out 1-based coordinates
                pos = col.reference_pos

                # If the column is not a cytosine in the reference, skip it
                if pos not in cs:
                    continue

                # Count the number of methylated and unmethylated aligned reads
                # by looping through each read in the "pileup"

                methylated = 0
                unmethylated = 0

                for read in col.pileups:
                    # If not present in alignment, skip
                    if (
                        read.alignment.query_sequence is None
                        or read.query_position is None
                    ):
                        continue

                    # Get the aligned base (should be either C or T)
                    base = read.alignment.query_sequence[read.query_position].upper()

                    # If base is a C, it was protected by methylation in EM process
                    if base == "C":
                        methylated += 1

                    # If base is a T, it was NOT protected by methylation
                    if base == "T":
                        unmethylated += 1

                # Compute statistics

                total = methylated + unmethylated

                if total > 0:
                    percent = f"{(methylated / total * 100):.5f}"
                else:
                    # If there were no reads at this position, we just print
                    # "NA" for percent
                    percent = "NA"

                # Write data

                f.write(
                    f"{pos + 1}\t{methylated}\t{unmethylated}\t{percent}\t{cs[pos]}\n"
                )


if __name__ == "__main__":
    if len(sys.argv) == 1:
        print(VERSION)
        print(f"\nFor help, run:\n\n    {PROG} -h\n")
        print(CITATION)
        sys.exit(0)

    parser = argparse.ArgumentParser(
        prog=PROG,
        description=call_methylation.__doc__,
    )

    parser.add_argument(
        "-v/--version",
        action="version",
        version=VERSION,
    )

    parser.add_argument(
        "--bam",
        required=True,
        help="BAM file to perform LEMON analysis on",
    )

    parser.add_argument(
        "--ref",
        required=True,
        help="FASTA reference file (original, not EM converted), must rename header with _CT_converted",
    )

    parser.add_argument(
        "--tsv",
        required=True,
        help="TSV file for output",
    )

    parser.set_defaults(
        func=lambda args: call_methylation(args.bam, args.ref, args.tsv)
    )

    # Dispatch command

    args = parser.parse_args()
    args.func(args)
