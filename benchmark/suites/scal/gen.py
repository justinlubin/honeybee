import glob
import os

MAX_BREADTH = 20
MAX_DEPTH = 20

for filename in glob.glob("b[0-9][0-9]d[0-9][0-9].hb"):
    os.remove(filename)

for breadth in range(1, MAX_BREADTH + 1):
    for depth in range(1, MAX_DEPTH + 1):
        with open(f"b{breadth:02d}d{depth:02d}.hb", "w") as f:
            f.write("(facts")
            for i in range(1, breadth + 1):
                f.write(f"\n  (In (.x {i}))")
            f.write(f")\n\n(goal\n  (Step{depth:02d}))\n")

with open("_suite.hblib", "w") as f:
    f.write("(input fact In (.x Int))\n\n")
    for depth in range(1, MAX_DEPTH + 1):
        f.write(f"(output fact Step{depth:02d})\n")
    f.write(f"\n(computation f01 Step01 ((in In)) ())\n")
    for depth in range(2, MAX_DEPTH + 1):
        f.write(
            f"(computation f{depth:02d} Step{depth:02d} ((in In) (s Step{depth-1:02d})) ())\n"
        )
