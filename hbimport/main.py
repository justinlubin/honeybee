import scrape
import step
import honeybee


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


def hbimport(url: str):
    # TODO: need to do goal inference
    pbn = honeybee.Controller(
        library="../editor/www/bio.hblib.toml",
        program="../editor/www/example.hb.toml",
    )

    ctx = scrape.PaperContext(url)
    decider = step.TraditionalStepDecider()

    while True:
        steps = [step.Step(s) for s in pbn.provide()]
        if not steps:
            break

        choice = decider.decide(ctx, steps)

        if choice is None:
            raise ValueError("Unsure between: " + " ".join(s.title for s in steps))

        # chosen_step = [s.title for s in steps if s.index == choice]
        # assert len(chosen_step) == 1
        # chosen_step = chosen_step[0]
        # print("Selection:", chosen_step)
        pbn.decide(choice)

    return pbn.working_expression()


e = hbimport("https://www.nature.com/articles/s41467-025-63167-x")
print(pretty(e))
