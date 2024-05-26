from dataclasses import dataclass

import pandas as pd
import altair as alt


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
        table: pd.DataFrame

    m: M
    d: D


@dataclass
class LibrarySizeHistogram:
    @dataclass
    class M:
        sample: str
        at: int

    @dataclass
    class D:
        pass

    m: M
    d: D


def bowtie(s: Seq) -> GeneQuantification.D:
    filepath = s.data
    command_output = ...  # filepath
    return pd.DataFrame(command_output)


def kallisto(s: Seq) -> GeneQuantification.D:
    filepath = s.data + s.at
    command_output = ...  # filepath
    return pd.DataFrame(columns=command_output)


def library_size_histogram(gq: GeneQuantification) -> LibrarySizeHistogram.D:
    alt.Chart(gq.d.table).interactive()
    return LibrarySizeHistogram.D()
