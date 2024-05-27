from dataclasses import dataclass

import pandas as pd
import altair as alt

# Sequencing


@dataclass
class Seq:
    sample: str
    at: int
    data: str


@dataclass
class GeneQuantification:
    @dataclass
    class M:
        sample: str
        at: int

    @dataclass
    class D:
        df: pd.DataFrame

    m: M
    d: D


def bowtie(s: Seq) -> GeneQuantification.D:
    df = ...
    return pd.DataFrame(df=df)


def kallisto(s: Seq) -> GeneQuantification.D:
    df = ...
    return pd.DataFrame(df=df)


# Transfection and growth


@dataclass
class Transfect:
    sample: str
    at: int
    library: str


@dataclass
class GrowthPhenotype:
    @dataclass
    class M:
        sample: str
        start: int
        end: int

    @dataclass
    class D:
        df: pd.DataFrame

    m: M
    d: D


def growth_phenotype(
    t: Transfect, q1: GeneQuantification, q2: GeneQuantification
) -> GrowthPhenotype.D:
    df = ...
    return GrowthPhenotype.D(df=df)


# Bulk RNA-seq


@dataclass
class ReadCountMatrix:
    @dataclass
    class M:
        sample1: str
        sample2: str
        at: int

    @dataclass
    class D:
        df: pd.DataFrame

    m: M
    d: D


@dataclass
class DifferentialGeneExpression:
    @dataclass
    class M:
        sample1: str
        sample2: str
        at: int

    @dataclass
    class D:
        df: pd.DataFrame

    m: M
    d: D


def load_read_counts(
    q1: GeneQuantification, q2: GeneQuantification
) -> ReadCountMatrix.D:
    df = ...
    return ReadCountMatrix.D(df=df)


def combat_seq(rcm: ReadCountMatrix) -> ReadCountMatrix.D:
    df = ...
    return ReadCountMatrix.D(df=df)


def deseq2(rcm: ReadCountMatrix) -> DifferentialGeneExpression.D:
    df = ...
    return DifferentialGeneExpression.D(df=df)
