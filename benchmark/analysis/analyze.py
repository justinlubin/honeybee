# %% Imports and monkey patching

import matplotlib.pyplot as plt
import numpy as np
import polars as pl

import matplotlib.figure

import os
import math


def save(self, filename, *args):
    os.makedirs(
        os.path.dirname(filename),
        exist_ok=True,
    )
    self.savefig(filename, *args)
    plt.close(self)


matplotlib.figure.Figure.save = save


def group_by_sel(self, by, *, sel):
    for (n,), g in self.group_by(by):
        yield n, g[sel]


pl.DataFrame.group_by_sel = group_by_sel

# %% Plotting functions


def distributions(
    groups,
    *,
    order,
    colors,
    bins,
    figsize,
    xlabel,
    ylabel="Count",
    total=None,
    flip=False,
    xticklabels=None,
):
    if xticklabels is not None:
        assert len(xticklabels) == len(bins)

    groups = sorted(groups, key=lambda x: order.index(x[0]))
    if flip:
        fig, ax = plt.subplots(
            1,
            3 * len(groups),
            gridspec_kw={"width_ratios": [1, 3, 1] * len(groups)},
            figsize=figsize,
            sharey=True,
        )
    else:
        fig, ax = plt.subplots(
            3 * len(groups),
            1,
            gridspec_kw={"height_ratios": [3, 1, 1] * len(groups)},
            figsize=figsize,
            sharex=True,
        )

    max_count = 0
    for i in range(0, len(groups)):
        (name, vals) = groups[i]
        color = colors[i]

        assert min(vals) >= min(bins)
        assert max(vals) <= max(bins)

        axa = ax[3 * i + 1] if flip else ax[3 * i]

        n, _, _ = axa.hist(
            vals,
            bins=bins,
            color=color,
            edgecolor="black",
            orientation="horizontal" if flip else "vertical",
        )
        max_count = max(max_count, max(n))

        if flip:
            axa.set_yticks(bins)
        else:
            axa.set_xticks(bins)

        axa.spines[["top", "right"]].set_visible(False)

        if name:
            axa.text(
                1,
                1,
                name,
                transform=axa.transAxes,
                color=color,
                ha="right",
                va="top",
                fontweight="bold",
                fontsize=16,
            )

        if total:
            if name:
                y = 0.93 if flip else 0.83
                text = f"({len(vals)}/{total} solved)"
            else:
                y = 1
                text = f"(on {len(vals)}/{total} entries)"

            axa.text(
                1,
                y,
                text,
                transform=axa.transAxes,
                color=color,
                ha="right",
                va="top",
                fontsize=10,
            )

        if flip:
            axa.set_xlabel(ylabel)
        else:
            axa.set_ylabel(ylabel)

        axb = ax[3 * i] if flip else ax[3 * i + 1]

        axb.boxplot(
            vals,
            vert=flip,
            widths=0.5,
            patch_artist=True,
            boxprops=dict(facecolor=color),
            medianprops=dict(color="black"),
        )

        if flip:
            axb.tick_params(left=True, labelleft=True)
            if xticklabels:
                axb.set_yticks(bins, labels=xticklabels)
            axb.spines[["right", "top", "bottom"]].set_visible(False)
            axb.get_xaxis().set_visible(False)
            axb.set_ylabel(xlabel)
        else:
            axb.tick_params(bottom=True, labelbottom=True)
            if xticklabels:
                axb.set_xticks(bins, labels=xticklabels)
            axb.spines[["right", "top", "left"]].set_visible(False)
            axb.get_yaxis().set_visible(False)
            axb.set_xlabel(xlabel)

        axc = ax[3 * i + 2]
        axc.set_visible(False)

    count_ticks = np.arange(0, max_count + 1, max(1, max_count // 4))
    for i in range(0, len(groups)):
        if flip:
            ax[3 * i + 1].set_xticks(count_ticks)
        else:
            ax[3 * i].set_yticks(count_ticks)

    fig.tight_layout()
    if flip:
        fig.subplots_adjust(wspace=0.1)
    else:
        fig.subplots_adjust(hspace=0.1)
    return fig, ax


def distribution(
    df,
    *,
    color,
    name=None,
    **kwargs,
):
    return distributions(
        [(name, df)],
        order=[name],
        colors=[color],
        **kwargs,
    )


def trapezoid(
    vals1,
    vals2,
    labels,
    *,
    xticklabel1,
    xticklabel2,
    ylabel,
    figsize,
    hpad=0.1,
    highlight_labels=set(),
):
    fig, ax = plt.subplots(1, 1, figsize=figsize)

    for v1, v2, label in zip(vals1, vals2, labels):
        color = "black"
        marker = "."
        zorder = 0

        if label in highlight_labels:
            color = "red"
            marker = "."
            zorder = 1

        ax.scatter([0], [v1], color=color, marker=marker, zorder=zorder)
        ax.scatter([1], [v2], color=color, marker=marker, zorder=zorder)

        ax.plot([0, 1], [v1, v2], color=color, zorder=zorder)

        ax.text(0 - hpad, v1, label, ha="right", va="center")
        ax.text(1 + hpad, v2, label, ha="left", va="center")

    ax.vlines(0, min(vals1), max(vals1), color="lightgray", zorder=-1)
    ax.vlines(1, min(vals2), max(vals2), color="lightgray", zorder=-1)

    ax.set_xticks(
        [-1, 0, 1, 2],
        labels=["", xticklabel1, xticklabel2, ""],
        fontweight="bold",
    )

    top = int(max(max(vals1), max(vals2))) + 1
    ax.set_yticks(np.arange(0, top + 1))
    ax.set_ylim(0, top)
    ax.set_ylabel(ylabel)

    ax.tick_params(bottom=False, labelbottom=True)
    ax.spines[["right", "top", "bottom"]].set_visible(False)

    fig.tight_layout()
    return fig, ax


def completion(vals, *, best, order, colors, figsize, xlabel):
    vals = sorted(vals, key=lambda x: order.index(x[0]))
    fig, ax = plt.subplots(1, 1, figsize=figsize)

    xticks = []
    xticklabels = []
    for i, (name, val) in enumerate(vals):
        r = ax.bar(i, val, color=colors[i])

        ax.bar_label(
            r,
            color=colors[i],
            fontsize=14,
        )

        xticks.append(i)
        xticklabels.append(name)

    ax.axhline(y=best, color="gray")
    ax.text(
        0.02,
        best + 0.2,
        f"Best possible: {best}",
        color="gray",
        transform=ax.get_yaxis_transform(),
    )

    ax.set_xticks(xticks, labels=xticklabels)
    ax.set_ylim(0, best + 1)

    ax.spines[["top", "right"]].set_visible(False)

    fig.tight_layout()
    return fig, ax


# %% Config

OUTPUT_DIR = "output"

ALGORITHMS = [
    "E",
    "EP",
    "PBN_E",
    "PBN_EP",
    "PBN_DL",
    "PBN_DLmem",
]

ALGORITHM_COLORS = [
    "#4477AA",
    "#66CCEE",
    "#228833",
    "#CCBB44",
    "#EE6677",
    "#AA3377",
]

# %% Load data

raw_data = pl.read_csv("../data/data.tsv", separator="\t")

# %% Process and check data

# Whether or not to perform validity checks of benchmark csv
CHECK = True

REPLICATE_KEY = ["suite", "entry", "task", "algorithm", "subentry"]

# Check that completed entries have at least one solution
if CHECK:
    df = raw_data.filter(pl.col("completed") & (pl.col("solution_count") == 0))
    with pl.Config(tbl_rows=-1, tbl_cols=-1):
        assert df.is_empty(), str(df)

# Check that completed replicates agree
if CHECK:
    for n, g in raw_data.filter(pl.col("completed")).group_by(REPLICATE_KEY):
        for c in ["solution_count", "solution_size"]:
            assert (g[c] == g[0, c]).all(), (n, c)

data = raw_data.group_by(REPLICATE_KEY).agg(
    duration=pl.col("duration").median() / 1000,
    completed=pl.col("completed").all(),
    solution_count=pl.col("solution_count").first(),
    solution_size=pl.col("solution_size").first(),
)

completed = data.filter(pl.col("completed")).drop("completed")

# Check that different approaches agree
if CHECK:
    for n, g in completed.group_by("suite", "entry", "task", "subentry"):
        if n[2] == "Any":
            continue
        for c in ["solution_count", "solution_size"]:
            assert (g[c] == g[0, c]).all(), (n, c, g)

solutions = (
    completed.filter(pl.col("task") == "All")
    .group_by(["suite", "entry"])
    .agg(count=pl.col("solution_count").first())
)

total_entries = {}
for (suite,), g in solutions.group_by("suite"):
    # TODO: Should simply be len(g) once every task has a solution
    entry_count = len(data.filter(pl.col("suite") == suite)["entry"].unique())
    total_entries[suite] = {
        "Any": entry_count,
        "All": entry_count,
        "Particular": g["count"].sum(),
    }

comparisons = (
    completed.join(
        completed,
        on=["suite", "entry", "task", "subentry"],
        suffix="2",
    )
    .filter(pl.col("algorithm") != pl.col("algorithm2"))
    .with_columns(speedup=pl.col("duration") / pl.col("duration2"))
)

# %% Plot summaries

for (suite, task), df in completed.group_by("suite", "task"):
    fig, ax = distributions(
        df.group_by_sel("algorithm", sel="duration"),
        total=total_entries[suite][task],
        order=ALGORITHMS,
        colors=ALGORITHM_COLORS,
        bins=np.arange(0, 30.1, 2),
        figsize=(20, 5),
        xlabel="Time taken (s)",
        flip=True,
    )

    fig.save(f"{OUTPUT_DIR}/{suite}/summary/{task}.pdf")

# %% Plot speedup comparisons


def pretty_abs_speedup(x, *, base, eps=1e-6):
    if x < 0:
        raise ValueError(f"impossible speedup: {x}")
    if abs(x - 1) < eps:
        return "1x\n"
    if x < 1:
        x = 1 / x
    return f"{int(x)}x"


def is_int_pow(x, *, base, eps=1e-6):
    log = math.log(x, base)
    return abs(log - round(log)) < eps


for (suite, task, alg1, alg2), df in comparisons.group_by(
    "suite", "task", "algorithm", "algorithm2"
):
    base = 2
    magnitude_lim = int(
        max(
            abs(math.log(df["speedup"].min(), base)),
            abs(math.log(df["speedup"].max(), base)),
        )
        + 1
    )
    magnitudes = np.arange(-magnitude_lim, magnitude_lim + 0.1, 1)
    bins = [base**m for m in magnitudes]
    fig, ax = distribution(
        df["speedup"],
        color=ALGORITHM_COLORS[0],
        total=total_entries[suite][task],
        bins=bins,
        figsize=(8, 3),
        xlabel="Speedup (log scale)",
    )

    ax[0].set_xscale("log", base=2)

    ax[1].set_xticks(
        bins,
        labels=[
            pretty_abs_speedup(b, base=base) if i % 1 == 0 else ""
            for i, b in enumerate(bins)
        ],
    )

    y = -0.66
    hpad = 0.05
    arrowprops = dict(
        facecolor="black",
        width=2,
        headwidth=6,
        headlength=4,
        shrink=0.1,
    )

    ax[1].annotate(
        alg1 + " faster",
        xy=(0, y),
        xytext=(hpad, y),
        xycoords=ax[1].transAxes,
        ha="left",
        va="center",
        fontweight="bold",
        arrowprops=arrowprops,
    )

    ax[1].annotate(
        alg2 + " faster",
        xy=(1, y),
        xytext=(1 - hpad, y),
        xycoords=ax[1].transAxes,
        ha="right",
        va="center",
        fontweight="bold",
        arrowprops=arrowprops,
    )

    for a in ax:
        a.axvline(x=1, color="gray", ls="dotted")
        # a.tick_params(axis="x", which="minor", bottom=False)

    fig.save(f"{OUTPUT_DIR}/{suite}/speedup/{task}-{alg1}-{alg2}.pdf")

# %% Plot space explored

for (suite,), df in (
    data.filter(pl.col("task") == "All")
    .join(solutions, on=["suite", "entry"])
    .group_by("suite")
):
    df = df.with_columns(explored=pl.col("solution_count") / pl.col("count"))

    fig, ax = distributions(
        df.group_by_sel("algorithm", sel="explored"),
        order=ALGORITHMS,
        colors=ALGORITHM_COLORS,
        bins=np.arange(0, 1.01, 0.1),
        figsize=(5, 15),
        xlabel="% explored",
    )

    fig.save(f"{OUTPUT_DIR}/{suite}/explored/explored.pdf")


# %% Plot trapezoids

for (suite, entry, alg1, alg2), df in comparisons.group_by(
    "suite", "entry", "algorithm", "algorithm2"
):
    df = df.group_by("task").agg(
        duration_med=pl.col("duration").median(),
        duration2_med=pl.col("duration2").median(),
    )
    fig, ax = trapezoid(
        df["duration_med"],
        df["duration2_med"],
        df["task"],
        xticklabel1=alg1,
        xticklabel2=alg2,
        ylabel="Time taken (s)",
        highlight_labels={"Particular"},
        figsize=(4, 3),
    )
    fig.save(f"{OUTPUT_DIR}/{suite}/trapezoid/{entry}-{alg1}-{alg2}.pdf")

# %% Plot completions

for (suite, task), df in completed.group_by("suite", "task"):
    fig, ax = completion(
        df.group_by("algorithm").len().rows(),
        best=total_entries[suite][task],
        order=ALGORITHMS,
        colors=ALGORITHM_COLORS,
        figsize=(5, 3),
        xlabel="Algorithm",
    )

    fig.save(f"{OUTPUT_DIR}/{suite}/completion/{task}.pdf")
