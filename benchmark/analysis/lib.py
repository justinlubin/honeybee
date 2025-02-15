import matplotlib.pyplot as plt
import numpy as np
import polars as pl

import matplotlib.figure

import os


def save(self, filename, *args, **kwargs):
    os.makedirs(
        os.path.dirname(filename),
        exist_ok=True,
    )
    self.savefig(filename, *args, **kwargs)
    plt.close(self)


matplotlib.figure.Figure.save = save


def show(df, sort_by=["algorithm", "entry_name"]):
    with pl.Config(tbl_cols=-1, tbl_rows=-1):
        print(df.sort(by=sort_by))


def distributions(
    df,
    *,
    group_feature,
    sort_feature,
    filter_feature,
    name_feature,
    value_feature,
    color_feature,
    count_feature,
    total_feature,
    bins,
    xlabel,
    ylabel="Count",
    flip=False,
    xticklabels=None,
):
    if xticklabels is not None:
        assert len(xticklabels) == len(bins)

    groups = list(
        df.filter(pl.col(filter_feature))
        .sort(sort_feature)
        .group_by(group_feature, maintain_order=True)
    )

    if flip:
        fig, ax = plt.subplots(
            1,
            3 * len(groups),
            gridspec_kw={"width_ratios": [1, 3, 1] * len(groups)},
            figsize=(4 * len(groups), 5),
            sharey=True,
        )
    else:
        fig, ax = plt.subplots(
            3 * len(groups),
            1,
            gridspec_kw={"height_ratios": [3, 1, 1] * len(groups)},
            figsize=(5, 4 * len(groups)),
            sharex=True,
        )

    max_bin_count = 0
    for i, (_, group) in enumerate(groups):
        name = group[name_feature][0]
        vals = group[value_feature]
        color = group[color_feature][0]
        count = group[count_feature][0]
        total = group[total_feature][0]

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
        max_bin_count = max(max_bin_count, max(n))

        if flip:
            axa.set_yticks(bins)
        else:
            axa.set_xticks(bins)

        axa.spines[["top", "right"]].set_visible(False)

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

        axa.text(
            0.5,
            1.03,
            f"({count}/{total} solved)",
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

    count_ticks = np.arange(0, max_bin_count + 1, max(1, max_bin_count // 4))
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


def speedup(
    df,
    *,
    left_value_feature,
    left_color_feature,
    left_name,
    left_short_name,
    right_value_feature,
    right_color_feature,
    right_name,
    right_short_name,
):
    better_left = df.filter(
        pl.col(left_value_feature) < pl.col(right_value_feature)
    )

    better_right = df.filter(
        pl.col(right_value_feature) < pl.col(left_value_feature)
    )

    fig, ax = plt.subplots(1, 1, figsize=(4, 4))

    ax.scatter(
        better_left[right_value_feature],
        better_left[left_value_feature],
        c=better_left[left_color_feature],
        zorder=2,
    )

    ax.scatter(
        better_right[right_value_feature],
        better_right[left_value_feature],
        c=better_right[right_color_feature],
        zorder=2,
    )

    max_duration = (
        int(
            max(
                df[left_value_feature].max(),
                df[right_value_feature].max(),
            )
        )
        + 1
    )

    ax.set_xlim([0, max_duration])
    ax.set_ylim([0, max_duration])

    ax.axline(xy1=(0, 0), slope=1, ls="--", c="lightgray", zorder=1)

    ax.set_xlabel(
        r"$\bf{" + right_name.replace(" ", r"\ ") + "}$" + "\nTime taken (s)"
    )
    ax.set_ylabel(
        r"$\bf{" + left_name.replace(" ", r"\ ") + "}$" + "\nTime taken (s)"
    )

    padding = 0.05

    ax.text(
        padding,
        1 - padding,
        r"$\bf{"
        + right_short_name
        + r"}$"
        + f" better ({len(better_right)}/{len(df)})",
        ha="left",
        va="top",
        transform=ax.transAxes,
    )

    ax.text(
        1 - padding,
        padding,
        r"$\bf{"
        + left_short_name
        + r"}$"
        + f" better ({len(better_left)}/{len(df)})",
        ha="right",
        va="bottom",
        transform=ax.transAxes,
    )

    ax.spines[["top", "right"]].set_visible(False)
    ax.set_aspect("equal", adjustable="box")

    fig.tight_layout()

    return fig, ax
