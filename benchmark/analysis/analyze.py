# %% Imports and monkey patching

import altair as alt
import matplotlib
import matplotlib.pyplot as plt
import numpy as np
import polars as pl

import os


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

# %% Setup

OUTPUT_DIR = "output"
raw_data = pl.read_csv("../data/data.csv")

# %% Config

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

TASKS = ["Particular", "Any", "All"]

# %% Data processing

KEY = ["entry", "task", "algorithm"]
REPLICATES = 0

for n, g in raw_data.filter(pl.col("completed")).group_by(KEY):
    if n[1] == "Particular":
        continue

    # Some replicates may not be completed
    REPLICATES = max(REPLICATES, len(g))

    # Check that replicates agree
    for c in ["solution_count", "solution_size"]:
        assert (g[c] == g[0, c]).all(), (n, c)

data = raw_data.group_by(KEY).agg(
    duration_med=pl.col("duration").median() / 1000,
    completed=pl.col("completed").all(),
)

entries = data["entry"].unique()

completed = data.filter(pl.col("completed")).drop("completed")

comparisons = (
    completed.join(completed, on=["entry", "task"], suffix="2")
    .filter(pl.col("algorithm") != pl.col("algorithm2"))
    .with_columns(
        log10_speedup=(pl.col("duration_med2") / pl.col("duration_med")).log10()
    )
)

solution_counts = {}

# = raw_data.filter(pl.col("completed") & (pl.algo))

for (entry,), g in raw_data.filter(
    pl.col("completed") & (pl.col("task") == "All")
).group_by("entry"):
    count = g[0, "solution_count"]
    size = g[0, "solution_size"]

    # Check that different approaches agree
    assert (g["solution_count"] == count).all(), (entry, c)
    assert (g["solution_size"] == size).all(), (entry, c)

    solution_counts[entry] = count


total_entries = {
    "Any": len(entries),
    "All": len(entries),
    "Particular": sum(solution_counts.values()),
}


# %% Summary distributions


def distributions(
    groups,
    *,
    total_entries,
    order,
    colors,
    bins,
    figsize,
    xlabel,
    ylabel="Count",
):
    groups = sorted(groups, key=lambda x: order.index(x[0]))
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

        axa = ax[3 * i]

        n, _, _ = axa.hist(
            vals,
            bins=bins,
            color=color,
            edgecolor="black",
        )
        max_count = max(max_count, max(n))

        axa.set_xticks(bins)
        axa.spines[["top", "right"]].set_visible(False)

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

        axa.text(
            1,
            0.83,
            f"({len(vals)}/{total_entries} solved)",
            transform=axa.transAxes,
            color=color,
            ha="right",
            va="top",
            fontsize=10,
        )

        axa.set_ylabel(ylabel)

        axb = ax[3 * i + 1]
        axb.boxplot(
            vals,
            vert=False,
            widths=0.5,
            patch_artist=True,
            boxprops=dict(facecolor=color),
            medianprops=dict(color="black"),
        )
        axb.tick_params(
            top=False, labeltop=False, bottom=True, labelbottom=True
        )
        axb.spines[["right", "top", "left"]].set_visible(False)
        axb.get_yaxis().set_visible(False)
        axb.set_xlabel(xlabel)

        axc = ax[3 * i + 2]
        axc.set_visible(False)

    for i in range(0, len(groups)):
        ax[3 * i].set_yticks(np.arange(0, max_count + 1, 1))

    fig.tight_layout()
    fig.subplots_adjust(hspace=0.1)
    return fig, ax


def distribution(
    df,
    *,
    color,
    name="",
    **kwargs,
):
    return distributions(
        [(name, df)],
        order=[name],
        colors=[color],
        **kwargs,
    )


for task in TASKS:
    fig, ax = distributions(
        data.filter(
            pl.col("completed") & (pl.col("task") == task)
        ).group_by_sel("algorithm", sel="duration_med"),
        total_entries=total_entries[task],
        order=ALGORITHMS,
        colors=ALGORITHM_COLORS,
        bins=np.arange(0, 30.1, 2),
        figsize=(5, 15),
        xlabel="Time taken (s)",
    )

    fig.save(f"{OUTPUT_DIR}/summary/{task}.pdf")

    for (alg1, alg2), df in comparisons.filter(pl.col("task") == task).group_by(
        ["algorithm", "algorithm2"]
    ):
        if alg1 != "PBN_EP" or alg2 != "PBN_DLmem":
            continue
        fig, ax = distribution(
            df["log10_speedup"],
            color=ALGORITHM_COLORS[0],
            total_entries=total_entries[task],
            bins=np.arange(-2, 2.1, 1),
            figsize=(5, 3),
            xlabel=r"$\log_{10}($Speedup$)$",
        )

        fig.save(f"{OUTPUT_DIR}/speedup/{task}-{alg1}-{alg2}.pdf")


# %% Trapezoid plots


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
            color = ALGORITHM_COLORS[0]
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


for (entry, alg1, alg2), df in comparisons.group_by(
    ["entry", "algorithm", "algorithm2"]
):
    if alg1 != "EP" or alg2 != "PBN_DLmem":
        continue
    fig, ax = trapezoid(
        df["duration_med"],
        df["duration_med2"],
        df["task"],
        xticklabel1=alg1,
        xticklabel2=alg2,
        ylabel="Time taken (s)",
        highlight_labels={"Particular"},
        figsize=(4, 3),
    )
    fig.save(f"{OUTPUT_DIR}/trapezoid/{entry}-{alg1}-{alg2}.pdf")

# %% Completion percentage


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


for task in TASKS:
    fig, ax = completion(
        data.filter(pl.col("completed") & (pl.col("task") == task))
        .group_by("algorithm")
        .len()
        .rows(),
        best=total_entries[task],
        order=ALGORITHMS,
        colors=ALGORITHM_COLORS,
        figsize=(5, 3),
        xlabel="Algorithm",
    )

    fig.save(f"{OUTPUT_DIR}/completion/{task}.pdf")


# %% Completion percentages

alt.Chart(data.reverse(), width=100).mark_bar(
    color="gray",
    stroke="black",
).encode(
    alt.X("algorithm:N").scale(domain=algorithms).title("Algorithm"),
    alt.Y("count()").title("Number solved").axis(grid=False),
    alt.Color("algorithm:N").scale(domain=algorithms, range=algorithm_colors),
    alt.Column("task:N", sort=tasks).title(None),
).transform_filter(
    alt.datum["completed"],
).configure_view(
    stroke=None,
).save(f"{OUTPUT}/completed.html")


# %% Histograms

ylabels = list(range(0, 9, 2))
ydomain = [min(ylabels), max(ylabels)]

xlabels = list(range(0, 16, 2))
xdomain = [min(xlabels), max(xlabels)]

for task in tasks:
    df = data.filter((pl.col("task") == task) & pl.col("completed"))
    chart = alt.vconcat()
    for algorithm, color in zip(algorithms, algorithm_colors):
        solved = (df.get_column("algorithm") == algorithm).sum()
        chart &= alt.vconcat(
            alt.Chart(
                title=f"{algorithm} (solved {solved}/{len(entries)})",
                height=70,
            )
            .mark_bar(
                color=color,
                stroke="black",
            )
            .encode(
                alt.X("duration_med:Q")
                .bin(step=2)
                .title(None)
                .axis(labels=False, values=xlabels)
                .scale(domain=xdomain),
                alt.Y("count()")
                .title("# Entries")
                .axis(grid=False, values=ylabels)
                .scale(domain=ydomain),
            ),
            alt.Chart()
            .mark_boxplot(
                color="black",
                box=alt.MarkConfig(color=color, stroke="black"),
                median=alt.MarkConfig(stroke="red"),
            )
            .encode(
                alt.X("duration_med:Q")
                .title("Duration (seconds)")
                .axis(grid=False, values=xlabels)
                .scale(domain=xdomain),
            ),
            data=df,
            spacing=0,
        ).transform_filter(alt.datum["algorithm"] == algorithm)

    chart.configure_view(
        stroke=None,
    ).save(f"{OUTPUT}/{task}.html")
