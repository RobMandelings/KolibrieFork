"""
For getting an overview that illustrates comparisons between different strategies across different workloads (e.g. varying window size)
While you focus on one parameter, such as throughput or memory consumption
"""
from typing import List

import pandas as pd
from matplotlib import pyplot as plt

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


def plot_overview(overview, ylabel, log_scale=False, strategies=None, title=None):
    if strategies is None:
        strategies = overview.index.tolist()

    x = list(overview.columns)

    # Choose different colors for strategies
    colors = {
        "clone": "tab:blue",
        "refcount": "tab:red",
        "arc": "tab:orange",
        "expire": "tab:green",
        "legacy": "tab:purple"
    }

    plt.figure(figsize=(16, 6))

    for strategy in overview.index:

        if strategy in strategies:
            y = overview.loc[strategy].values
            plt.plot(
                x,
                y,
                marker="o",
                label=strategy,
                color=colors.get(strategy, None),
            )

    plt.xlabel("window size label")  # or something more specific
    plt.xticks(rotation=45, ha="right", fontsize=6)
    plt.ylabel(ylabel)
    if log_scale:
        plt.yscale("log")
    if title is not None:
        plt.title(title)

    plt.legend()
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    plt.show()