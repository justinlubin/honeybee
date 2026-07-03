from dataclasses import dataclass

import honeybee


def title_lookup(text, steps):
    choice = None
    for s in steps:
        if s.title in text:
            choice = s.index
    return choice


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
    def applies(self, steps: list[Step]) -> bool: ...

    def decide(self, ctx: Context, steps: list[Step]) -> bool: ...


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

    return pbn.working_expression()
