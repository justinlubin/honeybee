import asyncio
import glob
import os
import random
import shutil
import sys

OPTION_COUNT = "option count: "
LIB_EXT = ".hblib.toml"
PROG_EXT = ".hb.toml"


async def run_one(hblib, prog, json):
    hb = await asyncio.create_subprocess_shell(
        " ".join(
            [
                "cargo",
                "run",
                "-q",
                "--",
                "interact",
                "--quiet",
                "-l",
                hblib,
                "-p",
                prog,
                "-j",
                json,
            ]
        ),
        stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE,
    )

    while True:
        stdout = (await hb.stdout.readline()).decode()
        if stdout.startswith(OPTION_COUNT):
            options = int(stdout[len(OPTION_COUNT) :])
        else:
            await hb.communicate()
            break
        choice = random.randint(1, options)

        hb.stdin.write((str(choice) + "\n").encode("utf-8"))
        await hb.stdin.drain()


async def run_all(suite, max_retries=30):
    hblib = suite + "/_suite" + LIB_EXT

    progs = sorted(glob.glob(suite + "/*" + PROG_EXT))

    for prog in progs:
        base = prog[: -len(PROG_EXT)]
        shutil.rmtree(base, ignore_errors=True)

    for prog in progs:
        random.seed(0)

        print(f"Working on '{prog}'... ", end="", flush=True)
        base = prog[: -len(PROG_EXT)]
        os.makedirs(base)
        outputs = set()
        for sample in range(1, N + 1):
            print(f"{sample}, ", end="", flush=True)
            json = f"{base}/sample{sample:05d}.json"
            for _ in range(max_retries):
                await run_one(hblib, prog, json)
                with open(json, "r") as f:
                    output = f.read()
                if output in outputs:
                    print("retry, ", end="", flush=True)
                else:
                    outputs.add(output)
                    break
            else:
                print("failed, ", end="")
                os.remove(json)
        print("done!")


if len(sys.argv) != 3:
    print(
        f"usage: python3 {sys.argv[0]} SUITE_NAME N_SAMPLES",
        file=sys.stderr,
    )
    sys.exit(1)

suite_name = sys.argv[1]
N = int(sys.argv[2])

os.chdir("../../backend/")
suite = "../benchmark/suites/" + suite_name
asyncio.run(run_all(suite))
