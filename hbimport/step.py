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


@dataclass
class TraditionalRule:
    step: set[str]
    method: set[str]
    fallback: set[str]

    def _step_trigger(self, steps: list[Step]) -> int | None:
        for sm in self.step:
            for s in steps:
                if sm in s.title:
                    return s.index
        return None

    def decide(self, steps: list[Step], *, methods: str) -> int | None:
        # Try to get the trigger
        st = self._step_trigger(steps)
        if st is None:
            return None

        # If there is a match, return the trigger
        for mm in self.method:
            if mm in methods:
                return st

        # Otherwise, try to get the fallback
        for f in self.fallback:
            for s in steps:
                if f in s.title:
                    return s.index

        return None


class TraditionalStepDecider(StepDecider):
    _RULES: list[TraditionalRule] = [
        TraditionalRule(
            step={"hg38", "HUMAN"},
            method={"human"},
            fallback={"OTHER"},
        ),
        TraditionalRule(
            step={"MultiQC"},
            method={""},
            fallback=set(),
        ),
        TraditionalRule(
            step={"remove poly(A)"},
            method={"poly(A)", "--poly-a"},
            fallback={"include poly(A)"},
        ),
    ]

    def _title_lookup(self, steps: list[Step], *, methods: str) -> int | None:
        for s in steps:
            if s.title in methods:
                return s.index
        return None

    @override
    def decide(self, ctx: scrape.PaperContext, steps: list[Step]) -> int | None:
        if len(steps) == 1:
            return 0

        joined_methods = "\n".join(ctx.methods())

        choice = self._title_lookup(steps, methods=joined_methods)
        if choice is not None:
            return choice

        for rule in self._RULES:
            choice = rule.decide(steps, methods=joined_methods)
            if choice is not None:
                return choice

        return None
