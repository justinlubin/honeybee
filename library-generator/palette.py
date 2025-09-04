# %%

from_sra_rna_seq(
    sra=SraRnaSeq(
        static=SraRnaSeq.S(label="hi", sample_sheet="playground/sra.csv"),
        dynamic=SraRnaSeq.D(),
    ),
    ret=RNASeq.S(label="hi", qc=False),
)

# %%
