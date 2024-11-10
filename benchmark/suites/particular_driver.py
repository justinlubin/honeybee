import asyncio
import glob
import os
import random
import shutil
import sys

random.seed(0)


async def run_one(hblib, py, prog, json):
    hb = await asyncio.create_subprocess_shell(
        " ".join(
            [
                "cargo",
                "run",
                "-q",
                "--",
                "run",
                "--mode",
                "process-interactive",
                "-l",
                hblib,
                "-i",
                py,
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
        try:
            options = int(stdout)
        except ValueError:
            await hb.communicate()
            break
        choice = random.randint(0, options - 1)

        hb.stdin.write((str(choice) + "\n").encode("utf-8"))
        await hb.stdin.drain()


async def run_all(suite, max_retries=3):
    hblib = suite + "/_suite.hblib"
    py = suite + "/_suite.py"

    progs = sorted(glob.glob(suite + "/*.hb"))

    for prog in progs:
        base = prog[:-3]
        shutil.rmtree(base, ignore_errors=True)

    for prog in progs:
        print(f"Working on '{prog}'... ", end="", flush=True)
        base = prog[:-3]
        os.makedirs(base)
        outputs = set()
        for sample in range(1, N + 1):
            print(f"{sample}, ", end="", flush=True)
            json = f"{base}/sample{sample:05d}.json"
            for _ in range(max_retries):
                await run_one(hblib, py, prog, json)
                with open(json, "r") as f:
                    output = f.read()
                if output in outputs:
                    print("retry, ", end="", flush=True)
                else:
                    outputs.add(output)
                    break
            else:
                print("failed, ", end="")
        print("done!")


suite_name = sys.argv[1]
N = int(sys.argv[2])

os.chdir("../../backend/")
suite = "../benchmark/suites/" + suite_name
asyncio.run(run_all(suite))
