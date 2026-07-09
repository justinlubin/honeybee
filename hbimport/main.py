from tqdm import tqdm
import scrape
import step
import honeybee


def pretty(e: dict) -> str:
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


def hbimport(url: str) -> dict | None:
    ctx = scrape.PaperContext(url)

    if "differential gene expression" in ctx.main_text():
        pbn = honeybee.Controller(
            library="../editor/www/bio.hblib.toml",
            program="../editor/www/example.hb.toml",
        )
    else:
        return None

    # decider = step.TraditionalStepDecider(ctx)
    decider = step.LlmStepDecider(
        ctx,
        model="lmstudio-community/Qwen3.5-0.8B-GGUF",
    )

    pbar = tqdm()
    while True:
        steps = [step.Step(s) for s in pbn.provide()]
        if not steps:
            break

        choice = decider.decide(steps)

        if choice is None:
            raise ValueError("Unsure between: " + ", ".join(s.title for s in steps))

        chosen_step = [s.title for s in steps if s.index == choice]
        assert len(chosen_step) == 1
        chosen_step = chosen_step[0]
        # print("Selection:", chosen_step)
        pbn.decide(choice)
        pbar.update()

    return pbn.working_expression()


for url in [
    "https://www.nature.com/articles/s41467-025-63167-x",
    "https://www.biorxiv.org/content/10.64898/2025.12.14.692434v1.full",
]:
    print("hbimporting", url)
    e = hbimport(url)
    if e is None:
        print("None")
    else:
        print(pretty(e))
