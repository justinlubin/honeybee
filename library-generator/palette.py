# type: ignore

# %%

Aret = RNASeq.S(label="hi", qc=False)
A = RNASeq(
    static=Aret,
    dynamic=from_sra_rna_seq(
        sra=SraRnaSeq(
            static=SraRnaSeq.S(label="hi", sample_sheet="playground/sra.csv"),
            dynamic=SraRnaSeq.D(),
        ),
        ret=Aret,
    ),
)

# %%

Bret = RNASeq.S(label="hi", qc=True)
B = RNASeq(
    static=Bret,
    dynamic=fastqc(
        data=A,
        ret=Bret,
    ),
)

# %%

Cret = RNASeq.S(label="hi", qc=True)
C = RNASeq(
    static=Cret,
    dynamic=multiqc(
        data=A,
        ret=Cret,
    ),
)

# %%

Dret = RNASeq.S(label="hi", qc=False)
D = RNASeq(
    static=Dret,
    dynamic=cutadapt_illumina(
        data=B,
        ret=Dret,
    ),
)
