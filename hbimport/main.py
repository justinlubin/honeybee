##############################################################################
# %% Imports
from typing import Self, override
import functools

import nodpy
from abc import ABC, abstractmethod

from dataclasses import dataclass

import re

import requests
from bs4 import BeautifulSoup

import honeybee


@functools.cache
def scrape(url: str) -> requests.Response:
    print("Cache miss for URL:", url)
    return requests.get(url)


def pretty(e):
    if "App" in e:
        fun = e["App"][0]["name"]
        args = []
        for param in e["App"][0]["arity"]:
            args.append(pretty(e["App"][1][param]))
        return fun + "(" + ", ".join(args) + ")"
    elif "Hole" in e:
        return "?" + str(e["Hole"])
    else:
        raise ValueError("Unknown expression")


# %%


@dataclass
class Step:
    title: str
    description: str
    info: dict
    index: int

    def __init__(self, step):
        self.title = step["function_title"]
        self.description = step["function_description"]
        self.info = step["info"]
        self.index = step["metadata_choices"][0]["choice_index"]


def title_lookup(text, steps):
    choice = None
    for s in steps:
        if s.title in text:
            choice = s.index
    return choice


class Extractor(ABC):
    _all: list[type[Extractor]] = []

    @override
    def __init_subclass__(cls: type[Extractor], /, **kwargs) -> None:
        super().__init_subclass__(**kwargs)
        Extractor._all.append(cls)

    @staticmethod
    def get_for(url: str) -> type[Extractor]:
        for cls in Extractor._all:
            if cls.matches(url):
                return cls
        raise ValueError("No matching extractor for URL:", url)

    @abstractmethod
    @staticmethod
    def matches(url: str) -> bool: ...

    @abstractmethod
    @staticmethod
    def methods(soup: BeautifulSoup) -> list[str]: ...

    @abstractmethod
    @staticmethod
    def prjna(soup: BeautifulSoup) -> str: ...


class NatureExtractor(Extractor):
    @override
    @staticmethod
    def matches(url: str) -> bool:
        return "nature.com" in url

    @override
    @staticmethod
    def methods(soup: BeautifulSoup) -> list[str]:
        methods = soup.find("section", attrs={"data-title": "Methods"})
        assert methods
        methods = methods.find_all("p")
        return [m.text for m in methods]

    @override
    @staticmethod
    def prjna(soup: BeautifulSoup) -> str:
        da = soup.find(
            "section",
            attrs={"data-title": "Data availability"},
        )
        assert da
        da = da.text
        assert da
        match = re.search("PRJNA[0-9]+", da)
        assert match
        return match.group(0)


class PaperContext:
    _main_url: str
    _extractor: type[Extractor]

    def __init__(self, url: str) -> None:
        self._url = url
        self._extractor = Extractor.get_for(url)

    @functools.cache
    def main_soup(self) -> BeautifulSoup:
        res = scrape(self._url)
        return BeautifulSoup(res.text, "html.parser")

    @functools.cache
    def methods(self) -> list[str]:
        return self._extractor.methods(self.main_soup())


class StepDecider(ABC):
    @abstractmethod
    def applies(self, steps: list[Step]) -> bool: ...

    def decide(self, ctx: Context, steps: list[Step]) -> bool: ...


def fastqc_lookup(methods, steps: list[Step]):
    choice = None
    for s in steps:
        if "FastQC" in s.title:
            choice = s.index
    return choice


def human_lookup(methods, steps: list[Step]):
    choice = None
    if "human" in methods:
        for s in steps:
            if "HUMAN" in s.title or "hg38" in s.title:
                choice = s.index
    else:
        for s in steps:
            if "OTHER" in s.title:
                choice = s.index
    return choice


def hbimport_nature(soup):
    methods = "".join(nature_methods(soup))
    prjna = nature_prjna(soup)

    # TODO: need to do goal inference
    pbn = honeybee.Controller(
        library="../editor/www/bio.hblib.toml",
        program="../editor/www/example.hb.toml",
    )

    while True:
        steps = [Step(s) for s in pbn.provide()]
        joined_steps = "".join([s.title for s in steps])

        if "hg38" in joined_steps:
            index = human_lookup(methods, steps)
        elif "FastQC" in joined_steps:
            index = fastqc_lookup(methods, steps)
        else:
            index = title_lookup(methods, steps)

        if index is None:
            print("unsure")
            print([s.title for s in steps])
            break

        print("selection:", steps[index].title)
        pbn.decide(index)

    nodpy.notebook()

    return pbn.working_expression()


def hbimport(paper_url):
    res = scrape(paper_url)
    soup = BeautifulSoup(res.text, "html.parser")
    if "nature.com" in paper_url:
        return hbimport_nature(soup)
    else:
        return None


e = hbimport("https://www.nature.com/articles/s41467-025-63167-x")
print(pretty(e))
