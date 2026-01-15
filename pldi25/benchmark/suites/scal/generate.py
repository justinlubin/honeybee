import glob
import os

# Constants

CONST_DEPTH = 5
CONST_BREADTH = 5

MIN_DEPTH = 1
MIN_BREADTH = 1

MAX_BREADTH = 10
MAX_DEPTH = 10

# Helpers


def make_entry(depth, breadth):
    with open(f"b{breadth:02d}d{depth:02d}.hb.toml", "w") as f:
        for i in range(1, breadth + 1):
            f.write("[[Prop]]\n")
            f.write('name = "In"\n')
            f.write(f"args.x = {i}\n\n")
        f.write("[Goal]\n")
        f.write(f'name = "Step{depth:02d}"\n')
        f.write("args = {}")


# Main

for filename in glob.glob("b[0-9][0-9]d[0-9][0-9].hb.toml"):
    os.remove(filename)

for depth in range(MIN_DEPTH, MAX_DEPTH + 1):
    make_entry(depth, CONST_BREADTH)

for breadth in range(MIN_BREADTH, MAX_BREADTH + 1):
    make_entry(CONST_DEPTH, breadth)

with open("_suite.hblib.toml", "w") as f:
    f.write('[Prop.In]\nparams.x = "Int"\n\n')
    f.write('[Type.Choice]\nparams.x = "Int"\n\n')

    for depth in range(1, MAX_DEPTH + 1):
        f.write(f"[Type.Step{depth:02d}]\n")
        f.write("params = {}\n\n")

    f.write("[Function.choose]\n")
    f.write("params = {}\n")
    f.write('ret = "Choice"\n')
    f.write('condition = ["In { x = ret.x }"]\n\n')

    f.write("[Function.f01]\n")
    f.write('params.choice = "Choice"\n')
    f.write('ret = "Step01"\n')
    f.write("condition = []\n")

    for depth in range(2, MAX_DEPTH + 1):
        f.write(f"\n[Function.f{depth:02d}]\n")
        f.write(f'params.s = "Step{depth - 1:02d}"\n')
        f.write('params.choice = "Choice"\n')
        f.write(f'ret = "Step{depth:02d}"\n')
        f.write("condition = []\n")
