# %%

import honeybee

pbn = honeybee.Controller(
    library="../notebook-editor/www/bio.hblib.toml",
    program="../notebook-editor/www/example.hb.toml",
    allow_undo=True,
)

solutions = set()


def exhaust():
    choices = pbn.provide()
    n = len(choices)
    if n == 0:
        solutions.add(str(pbn.working_expression()))
    for i in range(n):
        pbn.decide(i)
        exhaust()
        pbn.undo()


exhaust()

print("Solution count:", len(solutions))
