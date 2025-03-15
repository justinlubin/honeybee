import sys

if len(sys.argv) != 4:
    print(
        f"usage: uv run {sys.argv[0]} MAX_DURATION_SECONDS INPUT_TSV OUTPUT_DIR",
        file=sys.stderr,
    )
    sys.exit(1)

################################################################################
# % % Imports

import lib

import numpy as np
import polars as pl
import importlib

################################################################################
# % % Config and load

# Whether or not to use assertion checks
CHECK = False

# The synthesis cutoff duration, in milliseconds
MAX_DURATION_MS = int(sys.argv[1]) * 1000

# The CSV to load the data from
INPUT = sys.argv[2]

# The directory to output to
OUTPUT_DIR = sys.argv[3]

# The maximum breadth/depth for scalability analysis
MAX_DEPTH = 10
MAX_BREADTH = 10

# The breadth/depth that was held constant for scalability analysis
CONST_DEPTH = 5
CONST_BREADTH = 5

# The solution_name for the "any" task
ANY_TASK = "<ANY>"

# Key for algorithm's runs on a suite
SUITES = ["suite_name", "algorithm"]

# Key for sets of runs on same task
TASKS = SUITES + ["entry_name"]

# Key for sets of runs on same task with same particular solution (replicates)
REPLICATES = TASKS + ["solution_name"]

# Load metadata and data
raw_data = pl.read_csv(INPUT, separator="\t")
algorithm_metadata = pl.read_csv("algorithm_metadata.csv")

################################################################################
# % % Check and clean data

# Check that completed and successful columns are identical (no unsolvable
# problems)
if CHECK:
    assert (raw_data["completed"] == raw_data["success"]).all()

print(
    "note:",
    raw_data.select(
        pl.col("completed") & (pl.col("duration") > MAX_DURATION_MS)
    )
    .sum()
    .item(),
    "entries completed after time cutoff of",
    MAX_DURATION_MS // 1000,
    "seconds",
)

raw_data = raw_data.drop("success").with_columns(
    completed=pl.when(pl.col("duration") <= MAX_DURATION_MS)
    .then(raw_data["completed"])
    .otherwise(pl.lit(False))
)

################################################################################
# % % Process data

# Aggregate replicates
data = raw_data.group_by(REPLICATES).agg(
    duration=pl.col("duration").median() / 1000,
    completed=pl.col("completed").all(),
)

# Aggregate particulars
particulars = (
    data.filter(pl.col("solution_name") != ANY_TASK)
    .group_by(TASKS)
    .agg(
        duration=pl.col("duration").median(),
        completed=pl.col("completed").all(),
    )
)

# Collect anys
anys = data.filter(pl.col("solution_name") == ANY_TASK)

# Collect overall completion information (particular)
particular_overall_completion = particulars.group_by(SUITES).agg(
    overall_completed=pl.col("completed").sum(),
    total=pl.col("completed").len(),
)

# Collect overall completion information (any)
any_overall_completion = anys.group_by(SUITES).agg(
    overall_completed=pl.col("completed").sum(),
    total=pl.col("completed").len(),
)

# Compute scalability-specific columns
scal = (
    particulars.filter(pl.col("suite_name") == "scal")
    .with_columns(
        breadth=pl.col("entry_name").str.slice(1, 2).cast(int),
        depth=pl.col("entry_name").str.slice(4, 2).cast(int),
    )
    .drop("suite_name", "entry_name")
)

################################################################################
# % % Plot data

importlib.reload(lib)

### Summary plots

particular_summary = (
    particulars.filter(pl.col("completed"))
    .join(
        particular_overall_completion,
        how="left",
        on=["suite_name", "algorithm"],
        validate="m:1",
    )
    .join(
        algorithm_metadata,
        how="left",
        on="algorithm",
        validate="m:1",
    )
)

any_summary = (
    anys.filter(pl.col("completed"))
    .join(
        any_overall_completion,
        how="left",
        on=["suite_name", "algorithm"],
        validate="m:1",
    )
    .join(
        algorithm_metadata,
        how="left",
        on="algorithm",
        validate="m:1",
    )
)


def summary_plot(df):
    return lib.distributions(
        df,
        check=CHECK,
        group_feature="algorithm",
        sort_feature="algorithm_order",
        name_feature="algorithm_name",
        value_feature="duration",
        color_feature="algorithm_color",
        count_feature="overall_completed",
        total_feature="total",
        count_total_agg_feature="suite_name",
        bins=np.arange(0, 120.1, 2),
        xlabel="Time taken (s)",
        flip=True,
    )


# Fin

fig, ax = summary_plot(
    particular_summary.filter(
        (pl.col("suite_name") == "fin") & pl.col("algorithm_main"),
    )
)

fig.save(f"{OUTPUT_DIR}/01-fin.pdf")

# Inf

fig, ax = summary_plot(
    particular_summary.filter(
        (pl.col("suite_name") == "inf")
        & (pl.col("algorithm") == "PBNHoneybee"),
    )
)

fig.save(f"{OUTPUT_DIR}/02-inf.pdf")

# Any

fig, ax = summary_plot(
    any_summary.filter(
        (pl.col("suite_name").is_in(["fin", "inf"])) & pl.col("algorithm_main"),
    ),
)

fig.save(f"{OUTPUT_DIR}/05-any.pdf")

# Naive oracle, Fin

fig, ax = summary_plot(
    particular_summary.filter(
        (pl.col("suite_name") == "fin")
        & (pl.col("algorithm") == "PBNConstructiveOracle"),
    ),
)

# Appendix plot
# fig.save(f"{OUTPUT_DIR}/06-naive-fin.pdf")

# Naive oracle, Inf

fig, ax = summary_plot(
    particular_summary.filter(
        (pl.col("suite_name") == "inf")
        & (pl.col("algorithm") == "PBNConstructiveOracle"),
    ),
)

# Appendix plot
# fig.save(f"{OUTPUT_DIR}/07-naive-inf.pdf")

### Speedup plot

df = particulars.join(
    algorithm_metadata,
    how="left",
    on="algorithm",
    validate="m:1",
)

df = df.filter(
    (pl.col("algorithm") == "PBNHoneybee") & (pl.col("completed"))
).join(
    df.filter(
        (pl.col("algorithm") == "PBNHoneybeeNoMemo") & (pl.col("completed"))
    ),
    how="inner",
    on=["suite_name", "entry_name"],
    validate="1:1",
)

fig, ax = lib.speedup(
    df,
    left_value_feature="duration",
    left_color_feature="algorithm_color",
    left_name="Honeybee (Full)",
    left_short_name="Full",
    right_value_feature="duration_right",
    right_color_feature="algorithm_color_right",
    right_name="Honeybee (Ablation)",
    right_short_name="Ablation",
)

fig.save(f"{OUTPUT_DIR}/04-speedup.pdf")

### Scalability

fig, ax = lib.scalability(
    scal.filter(pl.col("completed"))
    .join(algorithm_metadata, how="left", on="algorithm", validate="m:1")
    .filter(pl.col("algorithm_main")),
    check=CHECK,
    max_breadth=MAX_BREADTH,
    max_depth=MAX_DEPTH,
    const_breadth=CONST_BREADTH,
    const_depth=CONST_DEPTH,
    group_feature="algorithm",
    sort_feature="algorithm_order",
    name_feature="algorithm_name",
    value_feature="duration",
    color_feature="algorithm_color",
    marker_feature="algorithm_marker",
    depth_feature="depth",
    breadth_feature="breadth",
)

fig.save(f"{OUTPUT_DIR}/03-scalability.pdf", bbox_inches="tight")
