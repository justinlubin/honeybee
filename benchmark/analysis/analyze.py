# %% Imports and monkey patching

import matplotlib.pyplot as plt
import numpy as np
import polars as pl

import matplotlib.figure

import os
import math


def save(self, filename, *args, **kwargs):
    os.makedirs(
        os.path.dirname(filename),
        exist_ok=True,
    )
    self.savefig(filename, *args, **kwargs)
    plt.close(self)


matplotlib.figure.Figure.save = save


def group_by_sel(self, by, *, sel):
    for (n,), g in self.group_by(by):
        yield n, g[sel]


pl.DataFrame.group_by_sel = group_by_sel


def assert_group_same(g, *, name, on):
    assert (g[on] == g[0, on]).all(), (name, on)


def show(df, sort_by=["algorithm", "entry"]):
    with pl.Config(tbl_cols=-1, tbl_rows=-1):
        print(df.sort(by=sort_by))


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

        assert min(vals) >= min(bins), (name, min(vals))
        assert max(vals) <= max(bins), (name, max(vals))

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
                0.5,
                1.05,
                name,
                transform=axa.transAxes,
                color=color,
                ha="center",
                va="bottom",
                fontweight="bold",
                fontsize=16,
            )

        if total:
            if name:
                text = f"({len(vals)}/{total} solved)"
            else:
                text = f"(on {len(vals)}/{total} entries)"

            axa.text(
                0.5,
                1.03,
                text,
                transform=axa.transAxes,
                color=color,
                ha="center",
                va="top",
                fontsize=14,
            )

        if flip:
            axa.set_xlabel(ylabel, fontweight="bold")
        else:
            axa.set_ylabel(ylabel, fontweight="bold")

        axb = ax[3 * i] if flip else ax[3 * i + 1]

        axb.boxplot(
            vals,
            vert=flip,
            widths=0.5,
            patch_artist=True,
            boxprops=dict(facecolor=color),
            medianprops=dict(color="black", lw=2),
        )

        if flip:
            axb.tick_params(left=True, labelleft=True)
            if xticklabels:
                axb.set_yticks(bins, labels=xticklabels)
            axb.spines[["right", "top", "bottom"]].set_visible(False)
            axb.get_xaxis().set_visible(False)
            axb.set_ylabel(xlabel, fontweight="bold")
        else:
            axb.tick_params(bottom=True, labelbottom=True)
            if xticklabels:
                axb.set_xticks(bins, labels=xticklabels)
            axb.spines[["right", "top", "left"]].set_visible(False)
            axb.get_yaxis().set_visible(False)
            axb.set_xlabel(xlabel, fontweight="bold")

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

DO_ANY = False

if DO_ANY:
    OUTPUT_DIR = "output-any"
else:
    OUTPUT_DIR = "output"

ALGOTASKS = [
    "E:All",
    "EP:All",
    "PBN_EP:Particular",
]

if DO_ANY:
    ALGOTASKS = [
        "E:Any",
        "EP:Any",
        "PBN_DLmem:Any",
    ]

APPROACH = [
    "Naïve Enumeration",
    "Pruned Enumeration",
    "Honeybee",
]

ALGORITHMS = [at.split(":")[0] for at in ALGOTASKS]

# https://personal.sron.nl/~pault/
ALGORITHM_COLORS = [
    "#66CCEE",
    "#4477AA",
    # "#228833",
    # "#CCBB44",
    # "#EE6677",
    "#AA3377",
]

CONST_DEPTH = 5
CONST_BREADTH = 5

MAX_DEPTH = 10
MAX_BREADTH = 10

# %% Load data

if DO_ANY:
    raw_data = pl.read_csv("../data/any.tsv", separator="\t")
else:
    raw_data = pl.read_csv("../data/particular.tsv", separator="\t")

# %% Process and check data

if DO_ANY:
    raw_data = raw_data.with_columns(suite=pl.lit("combined-ANY"))

# Whether or not to perform validity checks of benchmark csv
CHECK = True

ENTRY_KEY = ["suite", "entry", "task", "algorithm"]
SUBENTRY_KEY = ["suite", "entry", "task", "algorithm", "subentry"]

# Check that completed entries have at least one solution
if CHECK:
    df = raw_data.filter(pl.col("completed") & (pl.col("solution_count") == 0))
    with pl.Config(tbl_rows=-1, tbl_cols=-1):
        assert df.is_empty(), str(df)

# Check that completed subentries have all completed replicates
# if CHECK:
#     for n, g in raw_data.group_by(SUBENTRY_KEY):
#         assert_group_same(g, name=n, on="replicate")

# Check that completed replicates agree
if CHECK and not DO_ANY:
    for n, g in raw_data.filter(pl.col("completed")).group_by(SUBENTRY_KEY):
        for c in ["solution_count", "solution_size"]:
            assert_group_same(g, name=n, on=c)

data = raw_data.group_by(SUBENTRY_KEY).agg(
    duration=pl.col("duration").median() / 1000,
    completed=pl.col("completed").all(),
    solution_count=pl.col("solution_count").first(),
    solution_size=pl.col("solution_size").first(),
)

# Check that completed entries have all completed subentries
# if CHECK:
#     for n, g in raw_data.group_by(ENTRY_KEY):
#         assert_group_same(g, name=n, on="replicate")

# Aggregate particulars
aggdata = data.group_by(ENTRY_KEY).agg(
    duration=pl.col("duration").median(),
    completed=pl.col("completed").all(),
    solution_count=pl.col("solution_count").median(),
    solution_size=pl.col("solution_size").median(),
)

completed = aggdata.filter(pl.col("completed")).drop("completed")

total_entries = {}
for (suite,), g in raw_data.group_by("suite"):
    total_entries[suite] = g["entry"].unique().len()

comparisons = (
    completed.join(
        completed,
        on=["suite", "entry", "task"],
        suffix="2",
    )
    .filter(pl.col("algorithm") != pl.col("algorithm2"))
    .with_columns(speedup=pl.col("duration") / pl.col("duration2"))
)

scal = (
    aggdata.filter(pl.col("suite") == "scal")
    .with_columns(
        breadth=pl.col("entry").str.slice(1, 2).cast(int),
        depth=pl.col("entry").str.slice(4, 2).cast(int),
    )
    .drop("suite", "entry", "solution_count", "solution_size")
)

show(aggdata)

# %% Plot summaries

for (suite,), df in completed.filter(
    pl.col("algorithm").is_in(ALGORITHMS)
).group_by("suite"):
    if suite == "scal":
        continue
    if DO_ANY:
        assert (df["task"] == "Any").all()
    groups = sorted(
        [
            (APPROACH[ALGOTASKS.index(a + ":" + t)], g["duration"])
            for (a, t), g in df.group_by("algorithm", "task")
        ],
        key=lambda x: APPROACH.index(x[0]),
    )
    colors = [ALGORITHM_COLORS[APPROACH.index(at)] for at, _ in groups]
    fig, ax = distributions(
        groups,
        total=total_entries[suite],
        order=APPROACH,
        colors=colors,
        bins=np.arange(0, 40.1, 2),
        figsize=(4 * len(groups), 5),
        xlabel="Time taken (s)",
        flip=True,
    )

    fig.save(f"{OUTPUT_DIR}/{suite}/summary/{suite}-summary.pdf")

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


for (suite, task, alg1, alg2), df in comparisons.filter(
    (pl.col("algorithm") == "PBN_DL") & (pl.col("algorithm2") == "PBN_DLmem")
).group_by("suite", "task", "algorithm", "algorithm2"):
    base = 2

    try:
        magnitude_lim = int(
            max(
                abs(math.log(df["speedup"].min(), base)),
                abs(math.log(df["speedup"].max(), base)),
            )
            + 1
        )
    except ValueError:
        continue

    magnitudes = np.arange(-magnitude_lim, magnitude_lim + 0.1, 1)
    bins = [base**m for m in magnitudes]
    fig, ax = distribution(
        df["speedup"],
        color=ALGORITHM_COLORS[0],
        total=total_entries[suite],
        bins=bins,
        figsize=(9, 3),
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

    fig.save(f"{OUTPUT_DIR}/{suite}/speedup/{suite}-{task}-{alg1}-{alg2}.pdf")

# %% Plot speedup comparisons, v2

for (task, alg1, alg2), df in comparisons.filter(
    (pl.col("algorithm") == "PBN_DL") & (pl.col("algorithm2") == "PBN_DLmem")
    # & (pl.col("suite").is_in(["fin", "inf"]))
).group_by("task", "algorithm", "algorithm2"):
    approach1 = r"Honeybee\ (Ablation)"
    approach2 = r"Honeybee\ (Full)"
    approach1_short = "Ablation"
    approach2_short = "Full"
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
    max_duration = int(max(df["duration"].max(), df["duration2"].max())) + 1.5
    ax.set_xlim([0, max_duration])
    ax.set_ylim([0, max_duration])
    ax.axline(xy1=(0, 0), slope=1, ls="--", c="lightgray", zorder=1)
    ax.set_xlabel(r"$\bf{" + approach1 + "}$" + "\nTime taken (s)")
    ax.set_ylabel(r"$\bf{" + approach2 + "}$" + "\nTime taken (s)")
    padding = 0.05
    ax.text(
        padding,
        1 - padding,
        r"$\bf{" + approach1_short + "}$" + f" better ({better1}/{total})",
        ha="left",
        va="top",
        transform=ax.transAxes,
    )
    ax.text(
        1 - padding,
        padding,
        r"$\bf{" + approach2_short + "}$" + f" better ({better2}/{total})",
        ha="right",
        va="bottom",
        transform=ax.transAxes,
    )

    ax.spines[["top", "right"]].set_visible(False)
    ax.set_aspect("equal", adjustable="box")
    fig.tight_layout()

    fig.save(f"{OUTPUT_DIR}/overall-speedup/{task}-{alg1}-{alg2}.pdf")

    print("Median ablation speedup:", df["speedup"].median())

# %% Plot scalability

# Broken axis from:
#   https://matplotlib.org/stable/gallery/subplots_axes_and_figures/broken_axis.html

fig, ax = plt.subplots(
    2,
    2,
    figsize=(8, 5),
    sharex="col",
    sharey="row",
)

for i, (feature, other, const, x_max) in enumerate(
    [
        ("depth", "breadth", CONST_BREADTH, MAX_DEPTH),
        ("breadth", "depth", CONST_DEPTH, MAX_BREADTH),
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
        for row in range(0, 2):
            ax[row, i].plot(
                g[feature],
                g["duration"],
                c=ALGORITHM_COLORS[ati],
                marker=markers[j],
                label=APPROACH[ati] if i == 0 and row == 0 else None,
            )

    ax[1, i].spines[["top", "right"]].set_visible(False)
    # ax[i].set_aspect("equal", adjustable="box")
    featureUpper = feature[0].upper() + feature[1:]
    otherUpper = other[0].upper() + other[1:]
    ax[1, i].set_xlabel(
        r"$\bf{"
        + featureUpper
        + r"}$ $\bf{of}$ $\bf{search}$ $\bf{space}$"
        + f"\n(for {other} = {const})",
    )
    ax[1, i].set_ylabel(
        "Time taken (s)",
        fontweight="bold",
    )
    y_max = 12
    step = 2
    ax[1, i].set_xlim([0, x_max + 0.5])
    ax[1, i].set_ylim([0, y_max + step - 1])
    ax[1, i].set_xticks(np.arange(0, x_max + 0.1, 1))
    ax[1, i].set_yticks(np.arange(0, y_max + 0.1, step))
    ax[1, i].yaxis.set_tick_params(labelleft=True)

    outliers_min = 65
    outliers_max = outliers_min + y_max

    assert (
        df[feature].is_between(0, y_max + step - 1)
        | df[feature].is_between(outliers_min - (step - 1), outliers_max)
    ).all()

    ax[0, i].set_ylim([outliers_min - (step - 1), outliers_max])
    ax[0, i].set_yticks(np.arange(outliers_min, outliers_max + 0.1, step))
    ax[0, i].spines[["top", "right", "bottom"]].set_visible(False)
    ax[0, i].xaxis.set_tick_params(bottom=False)
    ax[0, i].yaxis.set_tick_params(labelleft=True)

    d = 0.5  # proportion of vertical to horizontal extent of the slanted line
    kwargs = dict(
        marker=[(-1, -d), (1, d)],
        markersize=12,
        linestyle="none",
        color="k",
        mec="k",
        mew=1,
        clip_on=False,
    )

    ax[0, i].plot([0], [0], transform=ax[0, i].transAxes, **kwargs)
    ax[1, i].plot([0], [1], transform=ax[1, i].transAxes, **kwargs)

fig.legend(ncol=3, loc="upper center", bbox_to_anchor=(0.5, 0))
fig.tight_layout()
fig.subplots_adjust(hspace=0.1)
fig.save(f"{OUTPUT_DIR}/scalability/scalability.pdf", bbox_inches="tight")

# %% ### OLD STUFF ###

import sys

sys.exit(0)

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
