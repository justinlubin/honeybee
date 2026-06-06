# %%

import re
import subprocess
from typing import Any

import jsonrpyc
import requests
from bs4 import BeautifulSoup

###############################################################################
# Monkey patching

original_request = jsonrpyc.Spec.request


@classmethod
def patched_request(
    cls,
    method: str,
    /,
    id: str | int | None = None,
    params: dict[str, Any] | None = None,
) -> str:
    if params is not None:
        params = params["args"]
    return original_request(method, id=id, params=params)


jsonrpyc.Spec.request = patched_request

###############################################################################
# Methods fetching + parsing

res = requests.get("https://www.nature.com/articles/s41467-025-63167-x")
soup = BeautifulSoup(res.text, "html.parser")

methods = soup.find("section", attrs={"data-title": "Methods"})
assert methods
methods = methods.find_all("p")
methods = [m.text for m in methods]
methods

###############################################################################
# JSON-RPC setup

p = subprocess.Popen(
    ["bash", "server.sh"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
)

assert p
assert p.stdin
assert p.stdout

rpc = jsonrpyc.RPC(stdout=p.stdin, stdin=p.stdout)  # type:ignore

###############################################################################
# Main loop

while True:
    steps: type[Any] = rpc.call("provide", block=0.1)  # type: ignore

    choices = {}
    for s in steps:
        choices[s["function_title"]] = s["metadata_choices"][0]["choice_index"]

    print(choices)

    choice = None
    for m in methods:
        for c in choices:
            if c in m:
                choice = c

    if choice is None:
        print("Unsure")
        break
    else:
        index = choices[choice]

    print("selection:", choice)
    rpc.call("decide", args=(index,), block=0.1)


print("Quitting:", rpc("quit", block=1))
p.stdin.close()
p.stdout.close()
p.terminate()

###############################################################################
# Get PRJNA


data_availability = soup.find("section", attrs={"data-title": "Data availability"})
assert data_availability is not None
da = data_availability.text
a = re.search("PRJNA[0-9]+", da)
assert a is not None
print(a.group(0))
