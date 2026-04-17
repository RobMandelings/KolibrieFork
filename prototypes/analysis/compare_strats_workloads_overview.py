"""
For getting an overview that illustrates comparisons between different strategies across different workloads (e.g. varying window size)
While you focus on one parameter, such as throughput or memory consumption
"""
from __future__ import annotations

from pathlib import Path
from typing import List

import pandas as pd
from matplotlib import pyplot as plt

from constants import STRATEGY_COLORS
from sorting import LabeledDataFrame


def build_overview_from_dfs(
        dfs: List[LabeledDataFrame],
        prop: str,
) -> pd.DataFrame:
    base_df = None

    for item in dfs:
        label = item.label
        df = item.dataframe

        col = df[[prop]].rename(columns={prop: label})

        if base_df is None:
            base_df = col
        else:
            base_df = base_df.join(col, how="outer")

    if base_df is None:
        return pd.DataFrame()

    return base_df


def plot_overview(overview, ylabel, strategies=None, title=None, output_file=None):
    if strategies is None:
        strategies = overview.index.tolist()

    x = list(overview.columns)

    plt.figure(figsize=(16, 6))

    for strategy in overview.index:

        if strategy in strategies:
            y = overview.loc[strategy].values
            plt.plot(
                x,
                y,
                marker="o",
                label=strategy,
                color=STRATEGY_COLORS.get(strategy, None),
            )

    plt.xlabel("window size label")  # or something more specific
    plt.xticks(rotation=45, ha="right", fontsize=6)
    plt.ylabel(ylabel)
    if title is not None:
        plt.title(title)

    plt.legend()
    plt.grid(True, alpha=0.3)
    plt.tight_layout()

    if output_file is not None:
        output_file = Path(output_file)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        plt.savefig(output_file, dpi=200, bbox_inches="tight")
        plt.close()
    else:
        plt.show()


def build_and_export_overviews(
        dfs: List[LabeledDataFrame],
        output_dir: str | Path
):
    """
    For each property/column, build an overview table across all LabeledDataFrames,
    then export a CSV and a plot.

    CSV path:   output_dir / f"{prop}.csv"
    Plot path:  output_dir / f"{prop}.png"
    """
    if not dfs:
        return

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Determine which properties to use
    first_df = dfs[0].dataframe
    # Use all columns from the first dataframe
    properties = first_df.columns.tolist()

    for prop in properties:
        if prop not in first_df.columns:
            # Skip properties that don't exist in the first df
            continue

        # 1) Build overview table for this property
        overview = build_overview_from_dfs(dfs, prop)

        # 2) Export CSV
        csv_path = output_dir / "csv" / f"{prop}.csv"
        # 3) Plot overview
        png_path = output_dir / "png" / f"{prop}.png"

        csv_path.parent.mkdir(parents=True, exist_ok=True)
        png_path.parent.mkdir(parents=True, exist_ok=True)
        overview.to_csv(csv_path)

        ylabel = prop  # or a nicer label mapping if you want
        plot_overview(
            overview=overview,
            ylabel=ylabel,
            title=prop,
            output_file=png_path,
        )
