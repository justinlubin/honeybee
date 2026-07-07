from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import override

import scrape


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


class StepDecider(ABC):
    @abstractmethod
    def decide(self, ctx: scrape.PaperContext, steps: list[Step]) -> int | None: ...


class TraditionalStepDecider(StepDecider):
    def _title_lookup(self, methods: str, steps: list[Step]) -> int | None:
        choice = None
        for s in steps:
            if s.title in methods:
                choice = s.index
        return choice

    def _fastqc_lookup(self, methods: str, steps: list[Step]) -> int | None:
        choice = None
        for s in steps:
            if "FastQC" in s.title:
                choice = s.index
        return choice

    def _human_lookup(self, methods: str, steps: list[Step]) -> int | None:
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

    @override
    def decide(self, ctx: scrape.PaperContext, steps: list[Step]) -> int | None:
        if len(steps) == 1:
            return 0

        joined_steps = "".join([s.title for s in steps])
        methods = "\n".join(ctx.methods())

        if "hg38" in joined_steps:
            return self._human_lookup(methods, steps)
        elif "FastQC" in joined_steps:
            return self._fastqc_lookup(methods, steps)
        else:
            return self._title_lookup(methods, steps)
