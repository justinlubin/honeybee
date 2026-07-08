import json
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import override

import ollama

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
            return steps[0].index

        joined_methods = "\n".join(ctx.methods())

        choice = self._title_lookup(steps, methods=joined_methods)
        if choice is not None:
            return choice

        for rule in self._RULES:
            choice = rule.decide(steps, methods=joined_methods)
            if choice is not None:
                return choice

        return None


class LlmStepDecider(StepDecider):
    _messages: list[ollama.Message]
    _model: str

    def __init__(self, *, model: str) -> None:
        self._messages = []
        self._model = model

    def _prompt(self, steps: list[Step], methods: str):
        return (
            methods
            + "\n\nWhich of the following computational steps should be used according to the above methods section?\n\n"
            + "\n".join(s.title for s in steps)
            + "\n\nRespond with ONLY the selected step. If unsure, choose a reasonable default among ONLY the provided steps."
        )

    @override
    def decide(self, ctx: scrape.PaperContext, steps: list[Step]) -> int | None:
        if len(steps) == 1:
            return steps[0].index

        step_titles = [s.title for s in steps]
        joined_methods = "\n".join(ctx.methods())

        prompt = self._prompt(steps, joined_methods)

        response = ollama.chat(
            model=self._model,
            messages=[{"role": "user", "content": prompt}],
            format={
                "type": "object",
                "properties": {"answer": {"type": "string", "enum": step_titles}},
                "required": ["answer"],
            },
            think=False,
        )

        if response.message.content is None:
            return None

        print(response.message.content)
        answer = json.loads(response.message.content)["answer"]
        print(answer, type(answer))

        return steps[step_titles.index(answer)].index

        # try:
        #     i = int(answer)
        # except ValueError:
        #     return None

        # if i < 0 or i >= len(steps):
        #     return None

        # return steps[i].index
