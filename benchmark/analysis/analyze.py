# %% Imports

import polars as pl
import altair as alt

# %% Constants

OUTPUT = "output"

# %% Data loading

raw = pl.read_csv("../data/data.csv")

# %% Data processing

raw_sucs = raw.filter(pl.col("completed"))

replicates = None

key = ["entry", "task", "algorithm"]

for n, g in raw_sucs.group_by(key):
    if replicates:
        assert len(g) == replicates, n
    else:
        replicates = len(g)

    for c in ["solution_count", "solution_size"]:
        vals = g.get_column(c)
        assert (vals == vals[0]).all(), (n, c)

data = raw.group_by(key).agg(
    duration_med=pl.col("duration").median() / 1000,
    completed=pl.col("completed").all(),
)

algorithms = ["PBN_Datalog", "PBN_DatalogMemo", "ALT_EnumPrune"]
algorithm_colors = ["antiquewhite", "lightgreen", "lightblue"]
tasks = ["Any", "All"]
entries = data.select(pl.col("entry")).unique()

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
