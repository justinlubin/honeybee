# hbimport

This directory contains "hbimport" infrastructure; code that acts as a
computer-led *step decider* (i.e., a *comptuational step provider*) for
Programming by Navigation.

The entry point for this code is `main.py` (you can run it with `uv run main`).

Some of the code is specific to scraping biology papers (`scrape.py`), but the
file `step.py` contains lots of reusable infrastructure for outside the biology
domain.

In particular:

- `Step` is a dataclass that contains information from the Programming by
  Navigation step provider (the Rust implementation) about a possible next step.
- `StepDecider` is the interface for step deciders. All step deciders need to do
  is implement `decide(self, steps: list[Step]) -> int`, which, given a list of
  steps, returns an index from `0` to `len(steps) - 1` that indicates the step
  to be chosen.

The file `step.py` contains two example computational step deciders:

- `TraditionalStepDecider` is a rule-based step decider that uses a set of
  heuristics encoded as `TraditionalRule`s to select between steps. If any of
  the rules "match", then it is used to select a step.
- `LllmStepDecider` uses an large language model (LLM) to select between the
  steps. The LLM is run **locally** using
  [llama.cpp](https://github.com/ggml-org/llama.cpp) via the
  [llama-cpp-python](https://github.com/abetlen/llama-cpp-python) Python
  library. `main.py` shows an example use of `LlmStepDecider` using Qwen 3.5
  0.8B, which runs quite quickly on a 2024 MacBook Pro.

In `main.py`'s `hbimport` function shows how to use an implementation of the
`StepDecider` interface. It's just a `while True` loop that calls `pbn.provide`
to get a list of steps, then `decider.decide` to select between them!

*Note:* The Python module `honeybee` is defined in the `engine` directory of
this repository. It uses PyO3 bindings to access the Rust implementation of
Programming by Navigation step providers. To make the Python bindings, you will
need the [`maturin`](https://github.com/PyO3/maturin) tool.
