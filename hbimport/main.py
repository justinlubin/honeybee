##############################################################################
# %% Imports


from bs4 import BeautifulSoup


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


# %%


def hbimport(paper_url):
    res = scrape(paper_url)
    soup = BeautifulSoup(res.text, "html.parser")
    if "nature.com" in paper_url:
        return hbimport_nature(soup)
    else:
        return None


e = hbimport("https://www.nature.com/articles/s41467-025-63167-x")
print(pretty(e))
