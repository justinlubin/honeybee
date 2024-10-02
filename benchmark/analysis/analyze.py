# %% Imports and monkey patching

import altair as alt
import matplotlib.pyplot as plt
import numpy as np
import polars as pl


def group_by_sel(self, by, sel):
    for (n,), g in self.group_by(by):
        yield n, g[sel]


pl.DataFrame.group_by_sel = group_by_sel

# %% Setup

OUTPUT_DIR = "output"
raw_data = pl.read_csv("../data/data.csv")


# %% Data processing

key = ["entry", "task", "algorithm"]
replicates = None

for n, g in raw_data.filter(pl.col("completed")).group_by(key):
    if n[1] == "Particular":
        continue

    if replicates:
        pass
        # assert len(g) == replicates, n
    else:
        replicates = len(g)

    for c in ["solution_count", "solution_size"]:
        assert (g[c] == g[0, c]).all(), (n, c)

data = raw_data.group_by(key).agg(
    duration_med=pl.col("duration").median() / 1000,
    completed=pl.col("completed").all(),
)

algorithms = [
    "E",
    "EP",
    "PBN_E",
    "PBN_EP",
    "PBN_DL",
    "PBN_DLmem",
]

algorithm_colors = [
    "#4477AA",
    "#66CCEE",
    "#228833",
    "#CCBB44",
    "#EE6677",
    "#AA3377",
]

tasks = ["Particular", "Any", "All"]
entries = data["entry"].unique()

# %%


def distributions(groups, *, order, colors, bins):
    groups = sorted(groups, key=lambda x: order.index(x[0]))
    fig, ax = plt.subplots(
        2 * len(groups),
        1,
        gridspec_kw={"height_ratios": [2, 1] * len(groups)},
        figsize=(5, 15),
        sharex=True,
    )

    max_count = 0
    for i in range(0, len(groups)):
        (name, vals) = groups[i]
        color = colors[i]

        axa = ax[2 * i]

        n, _, _ = axa.hist(vals, bins=bins, color=color)
        max_count = max(max_count, max(n))

        axa.set_xticks(bins)
        axa.spines[["top", "right"]].set_visible(False)

        axb = ax[2 * i + 1]
        axb.boxplot(
            vals,
            vert=False,
        )
        axb.tick_params(
            top=True, labeltop=True, bottom=False, labelbottom=False
        )
        axb.spines[["right", "bottom", "left"]].set_visible(False)

    for i in range(0, len(groups)):
        ax[2 * i].set_yticks(np.arange(0, max_count, 1))

    fig.tight_layout()
    return fig


distributions(
    data.filter(pl.col("task") == "Particular").group_by_sel(
        "algorithm", "duration_med"
    ),
    order=algorithms,
    colors=algorithm_colors,
    bins=np.arange(0, 30, 2),
).savefig(f"{OUTPUT_DIR}/particular.pdf")

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
