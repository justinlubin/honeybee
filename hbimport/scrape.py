import functools
import re
from abc import ABC, abstractmethod
from typing import override, Iterator
import base64

import requests
from bs4 import BeautifulSoup, Tag


class UrlCache:
    _filename: str
    _cache: dict[str, str]

    def __init__(self, filename: str) -> None:
        self._filename = filename
        self._cache = {}

        try:
            # Fill in-memory cache from file cache
            with open(self._filename, "r") as f:
                for line in f:
                    url, encoded_result = line.split("\t", maxsplit=1)
                    result = base64.b64decode(encoded_result).decode("utf-8")
                    self._cache[url] = result
        except FileNotFoundError:
            pass

    def scrape(self, url: str) -> str:
        # Check in-memory cache
        if url in self._cache:
            return self._cache[url]

        print("Cache miss for URL", url)

        # Make HTTP request
        result = requests.get(url).text

        # Save result to in-memory cache
        self._cache[url] = result

        # Save result to file cache
        encoded_result = base64.b64encode(result.encode("utf-8")).decode("utf-8")
        with open(self._filename, "a") as f:
            f.write(f"{url}\t{encoded_result}\n")

        return result


GLOBAL_URL_CACHE = UrlCache("cache.tsv")


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

    @staticmethod
    @abstractmethod
    def matches(url: str) -> bool: ...

    @staticmethod
    @abstractmethod
    def methods(soup: BeautifulSoup) -> list[str]: ...

    @staticmethod
    @abstractmethod
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
        match = re.search("PRJNA[0-9]+", da)
        assert match
        return match.group(0)


class BioRxivExtractor(Extractor):
    _METHOD_RE = re.compile("Method")

    @override
    @staticmethod
    def matches(url: str) -> bool:
        return "biorxiv.org" in url

    @override
    @staticmethod
    def methods(soup: BeautifulSoup) -> list[str]:
        methods = soup.find("h2", string=BioRxivExtractor._METHOD_RE)  # type: ignore
        assert isinstance(methods, Tag)
        methods = methods.parent
        assert methods
        methods = methods.find_all("p")
        return [m.text for m in methods]

    @override
    @staticmethod
    def prjna(soup: BeautifulSoup) -> str:
        match = re.search("PRJNA[0-9]+", str(soup))
        assert match
        return match.group(0)


class PaperContext:
    _main_url: str
    _extractor: type[Extractor]

    def __init__(self, url: str) -> None:
        self._url = url
        self._extractor = Extractor.get_for(url)

    def main_text(self) -> str:
        return GLOBAL_URL_CACHE.scrape(self._url)

    @functools.cache
    def main_soup(self) -> BeautifulSoup:
        return BeautifulSoup(self.main_text(), "html.parser")

    @functools.cache
    def methods(self) -> list[str]:
        return self._extractor.methods(self.main_soup())


def _biorxiv_paper_listing_url(
    *,
    start: str,
    end: str,
    cursor: int,
    category: str | None,
):
    assert cursor >= 0
    # return f"https://api.biorxiv.org/pub/{start}/{end}/{cursor}"
    url = f"https://api.biorxiv.org/details/biorxiv/{start}/{end}/{cursor}/json"
    if category is not None:
        url += "?category=" + category
    return url


def _biorxiv_get_url(doi: str) -> str:
    return f"https://www.biorxiv.org/content/{doi}.full"


def biorxiv_paper_urls(
    *,
    start: str,
    end: str,
    category: str | None = None,
    max_papers: int | None = None,
) -> Iterator[str]:
    paper_count = 0
    cursor = 0
    while True:
        # Note: Sends an uncached request bioRxiv!
        res = requests.get(
            _biorxiv_paper_listing_url(
                start=start,
                end=end,
                cursor=cursor,
                category=category,
            )
        ).json()

        if res["collection"] == []:
            break
        for paper in res["collection"]:
            if max_papers is not None and paper_count >= max_papers:
                return

            yield _biorxiv_get_url(paper["doi"])

            paper_count += 1

        cursor += 1
