################################################################################
# % % Imports

import matplotlib.pyplot as plt
import numpy as np
import polars as pl
import importlib

import lib

################################################################################
# % % Config and load

# The directory to output to
OUTPUT_DIR = "output"

# Whether or not to perform validity checks of benchmark csv
CHECK = True

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
raw_data = pl.read_csv("../data/data.tsv", separator="\t")
algorithm_metadata = pl.read_csv("algorithm_metadata.csv")

################################################################################
# % % Check data

if CHECK:
    # Check that completed and successful columns are identical (no unsolvable
    # problems)
    assert (raw_data["completed"] == raw_data["success"]).all()

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

# Collect completed tasks
completed = particulars.filter(pl.col("completed")).drop("completed")

# Collect overall completion information
overall_completion = anys.group_by(SUITES).agg(
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
# %% Plot data

importlib.reload(lib)

# Main summary plots

for (suite,), df in (
    completed.join(
        overall_completion,
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
    .group_by("suite_name")
):
    fig, ax = lib.distributions(
        df,
        group_feature="algorithm",
        filter_feature="algorithm_main",
        sort_feature="algorithm_order",
        name_feature="algorithm_name",
        value_feature="duration",
        color_feature="algorithm_color",
        count_feature="overall_completed",
        total_feature="total",
        bins=np.arange(0, 10.1, 1),
        xlabel="Time taken (s)",
        flip=True,
    )

    fig.save(f"{OUTPUT_DIR}/{suite}.pdf")

# Speedup plots

particulars_m = particulars.join(
    algorithm_metadata,
    on=["algorithm"],
    how="left",
    validate="m:1",
)

df = particulars_m.filter(pl.col("algorithm") == "PBNHoneybee").join(
    particulars_m.filter(pl.col("algorithm") == "PrunedEnumeration"),
    how="inner",
    on=["suite_name", "entry_name"],
    validate="1:1",
)

fig, ax = lib.speedup(df, left_value_feature="duration", left_value_name)

# %%

# fig, ax = lib.speedup(


approach1 = "Ablation"
approach2 = "Full"
total = len(df)
better1 = len(df.filter(pl.col("duration") < pl.col("duration2")))
better2 = len(df.filter(pl.col("duration2") < pl.col("duration")))
fig, ax = plt.subplots(1, 1, figsize=(4, 4))
ax.scatter(
    df["duration"],
    df["duration2"],
    zorder=2,
    c=ALGORITHM_COLORS[2],
    # edgecolor="black",
    # linewidth=0.5,
)
max_duration = int(max(df["duration"].max(), df["duration2"].max())) + 1
ax.set_xlim([0, max_duration])
ax.set_ylim([0, max_duration])
ax.axline(xy1=(0, 0), slope=1, ls="--", c="lightgray", zorder=1)
ax.set_xlabel(r"$\bf{" + approach1 + "}$" + "\nTime taken (s)")
ax.set_ylabel(r"$\bf{" + approach2 + "}$" + "\nTime taken (s)")
padding = 0.05
ax.text(
    padding,
    1 - padding,
    r"$\bf{" + approach1 + "}$" + f" better ({better1}/{total})",
    ha="left",
    va="top",
    transform=ax.transAxes,
)
ax.text(
    1 - padding,
    padding,
    r"$\bf{" + approach2 + "}$" + f" better ({better2}/{total})",
    ha="right",
    va="bottom",
    transform=ax.transAxes,
)

ax.spines[["top", "right"]].set_visible(False)
ax.set_aspect("equal", adjustable="box")
fig.tight_layout()

fig.save(f"{OUTPUT_DIR}/overall-speedup/{task}-{alg1}-{alg2}.pdf")

# %% Plot scalability

fig, ax = plt.subplots(
    1,
    2,
    figsize=(8, 4),
    sharey=True,
)

for i, (feature, other, const) in enumerate(
    [
        ("depth", "breadth", CONST_BREADTH),
        ("breadth", "depth", CONST_DEPTH),
    ]
):
    df = scal.filter(
        (pl.col(other) == const)
        & pl.col("completed")
        & pl.col("algorithm").is_in(ALGORITHMS)
    ).sort(by=feature)

    groups = sorted(
        df.group_by("algorithm", "task"),
        key=lambda x: ALGOTASKS.index(x[0][0] + ":" + x[0][1]),
    )

    markers = ["s", "^", "o"]
    for j, ((a, t), g) in enumerate(groups):
        ati = ALGOTASKS.index(a + ":" + t)
        ax[i].plot(
            g[feature],
            g["duration"],
            c=ALGORITHM_COLORS[ati],
            marker=markers[j],
            label=APPROACH[ati] if i == 0 else None,
        )

    ax[i].spines[["top", "right"]].set_visible(False)
    ax[i].set_aspect("equal", adjustable="box")
    featureUpper = feature[0].upper() + feature[1:]
    otherUpper = other[0].upper() + other[1:]
    ax[i].set_xlabel(
        r"$\bf{"
        + featureUpper
        + r"}$ $\bf{of}$ $\bf{search}$ $\bf{space}$"
        + f"\n(for {other} = {const})",
    )
    ax[i].set_ylabel(
        "Time taken (s)",
        fontweight="bold",
    )
    ax[i].set_xlim([0, 10.5])
    ax[i].set_ylim([0, 10.5])
    ax[i].set_xticks(np.arange(0, 10.1, 1))
    ax[i].set_yticks(np.arange(0, 10.1, 1))
    ax[i].yaxis.set_tick_params(labelleft=True)

fig.legend(ncol=3, loc="upper center", bbox_to_anchor=(0.5, 0))
fig.tight_layout()
fig.save(f"{OUTPUT_DIR}/scalability/scalability.pdf", bbox_inches="tight")
