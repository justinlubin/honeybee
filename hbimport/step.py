import json
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import override, Iterator, Callable, Literal
import warnings

from llama_cpp import (
    Llama,
    CreateChatCompletionResponse,
    ChatCompletionRequestResponseFormat,
)

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
    def decide(self, steps: list[Step]) -> int | None: ...


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

    _ctx: scrape.PaperContext

    def __init__(self, ctx: scrape.PaperContext) -> None:
        self._ctx = ctx

    def _title_lookup(self, steps: list[Step], *, methods: str) -> int | None:
        for s in steps:
            if s.title in methods:
                return s.index
        return None

    @override
    def decide(self, steps: list[Step]) -> int | None:
        if len(steps) == 1:
            return steps[0].index

        joined_methods = "\n".join(self._ctx.methods())

        choice = self._title_lookup(steps, methods=joined_methods)
        if choice is not None:
            return choice

        for rule in self._RULES:
            choice = rule.decide(steps, methods=joined_methods)
            if choice is not None:
                return choice

        return None


class LlmStepDecider(StepDecider):
    _ctx: scrape.PaperContext
    _llm: Llama
    _messages: list

    def __init__(self, ctx: scrape.PaperContext, *, model: str) -> None:
        self._ctx = ctx
        self._messages = []

        with warnings.catch_warnings():
            warnings.filterwarnings(
                "ignore",
                message="The `local_dir_use_symlinks` argument is deprecated",
            )
            self._llm = Llama.from_pretrained(
                repo_id=model,
                filename="*Q8_0.gguf",
                n_gpu_layers=-1,
                n_ctx=8192,
                verbose=False,
            )

        joined_methods = "\n".join(self._ctx.methods())

        self._messages.append(
            {
                "role": "system",
                "content": "You will be provided with the methods section of a paper and subsequently asked about which computational steps should be used according to the methods section. For now, here is the methods section: "
                + joined_methods,
            }
        )

    def _chat(
        self,
        prompt: str,
        *,
        response_format: ChatCompletionRequestResponseFormat | None = None,
    ) -> str:
        self._messages.append({"role": "user", "content": prompt})
        response = self._llm.create_chat_completion(
            messages=self._messages,
            response_format=response_format,
        )
        assert not isinstance(response, Iterator)

        content = response["choices"][0]["message"]["content"]
        assert content is not None

        self._messages.append({"role": "assistant", "content": content})

        return content

    def _make_schema(self, options: list[str]) -> ChatCompletionRequestResponseFormat:
        return {
            "type": "json_object",
            "schema": {
                "type": "string",
                "enum": options,
            },
        }

    # def _prompt(self, steps: list[Step], methods: str):
    #     return (
    #         # methods
    #         # + "\n\nWhich of the following computational steps should be used according to the above methods section?\n\n"
    #         # + "\n".join(s.title for s in steps)
    #         # + "\n\nRespond with ONLY the selected step. If unsure, choose a reasonable default among ONLY the provided steps."
    #         methods
    #         + "\n\nWhich computational step should be used according to the above methods section?\n"
    #     )

    @override
    def decide(self, steps: list[Step]) -> int | None:
        if len(steps) == 1:
            return steps[0].index

        step_titles = [s.title for s in steps]

        response = self._chat(
            "Which computational step should be used according to the above methods section?\n",
            response_format=self._make_schema(step_titles),
        )

        answer = json.loads(response)

        return steps[step_titles.index(answer)].index
