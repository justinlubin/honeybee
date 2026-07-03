import functools
import re
from abc import ABC, abstractmethod
from typing import override

import requests
from bs4 import BeautifulSoup


@functools.cache
def scrape(url: str) -> requests.Response:
    print("Cache miss for URL:", url)
    return requests.get(url)


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
