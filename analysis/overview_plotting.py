from pathlib import Path
from typing import Any, Callable

import pandas as pd
from matplotlib import pyplot as plt

from constants import STRATEGY_COLORS, STRATEGY_MARKERS
from series import linear_trend_per_strategy_df, add_regression_overlay


def make_overview_plotter(
        *,
        y_col: str,
        title: str,
        xlabel: str,
        ylabel: str,
        workload_index_col: str,
        output_file,
        label_fn,
        strategies=None,
        descending: bool = False,
        x_label_col: str = "x_label",
        strategy_col: str = "strategy",
        yerr_col=None,
):
    def plotter(df: pd.DataFrame, analysis_path):
        print(f"Plotting to output: {output_file}")
        plot_overview_from_df(
            df=df,
            analysis_path=analysis_path,
            y_col=y_col,
            title=title,
            xlabel=xlabel,
            ylabel=ylabel,
            workload_index_col=workload_index_col,
            output_file=output_file,
            label_fn=label_fn,
            strategies=strategies,
            descending=descending,
            x_label_col=x_label_col,
            strategy_col=strategy_col,
            yerr_col=yerr_col,
        )

    return plotter


def add_label_column(
        df: pd.DataFrame,
        label_fn: Callable[[pd.Series], Any],
        out_col: str = "x_label",
) -> pd.DataFrame:
    """
    Add a label column computed from each row using `label_fn`.

    Parameters
    ----------
    df : pd.DataFrame
        Input dataframe.
    label_fn : callable
        Function that accepts a row (pd.Series) and returns the label.
    out_col : str
        Name of the output column to create.

    Returns
    -------
    pd.DataFrame
        Copy of df with the new column added.
    """
    df = df.copy()
    df[out_col] = df.apply(label_fn, axis=1)
    return df


OVERLAY_REGRESSION = False


def plot_overview_from_df(
        df,
        analysis_path: Path,
        label_fn: Callable[[pd.Series], Any],
        y_col: str,
        xlabel: str,
        ylabel: str,
        workload_index_col,
        strategies=None,
        title=None,
        output_file=None,
        x_label_col="x_label",
        strategy_col="strategy",
        yerr_col=None,
        descending: bool = False,
):
    df = add_label_column(df, label_fn=label_fn, out_col=x_label_col)
    df = df.copy()

    if strategies is None:
        strategies = list(df[strategy_col].dropna().unique())

    workload_order = (
        df[workload_index_col]
            .drop_duplicates()
            .sort_values(ascending=not descending)
            .tolist()
    )

    x_pos = list(range(len(workload_order)))
    workload_index_to_x = {w: i for i, w in enumerate(workload_order)}

    x_tick_labels = []
    for workload_idx in workload_order:
        subset = df[df[workload_index_col] == workload_idx]
        if subset.empty:
            x_tick_labels.append(str(workload_idx))
        elif x_label_col in subset.columns:
            x_tick_labels.append(str(subset.iloc[0][x_label_col]))
        else:
            x_tick_labels.append(str(workload_idx))

    fig, ax1 = plt.subplots(figsize=(16, 6))

    if OVERLAY_REGRESSION:
        regression_results = linear_trend_per_strategy_df(
            df=df,
            y_col=y_col,
            strategy_col=strategy_col,
            descending=descending,
            x_order_col="window.slide",
        )
    else:
        regression_results = None

    for strategy in strategies:
        strat_df = df[df[strategy_col] == strategy].copy()
        if strat_df.empty:
            continue

        strat_df = strat_df[strat_df[workload_index_col].isin(workload_order)]
        strat_df = strat_df.sort_values(by=workload_index_col)

        cur_x_pos = []
        y_values = []
        y_errors = []
        has_any_error = False

        for workload_idx in workload_order:
            row_df = strat_df[strat_df[workload_index_col] == workload_idx]
            if row_df.empty:
                continue

            row = row_df.iloc[0]

            cur_x_pos.append(workload_index_to_x[workload_idx])
            y_values.append(row[y_col])

            if yerr_col is not None and yerr_col in row_df.columns and pd.notna(row[yerr_col]):
                y_errors.append(row[yerr_col])
                has_any_error = True
            else:
                y_errors.append(None)

        if not cur_x_pos:
            continue

        marker = STRATEGY_MARKERS.get(strategy, "o")
        color = STRATEGY_COLORS.get(strategy, None)

        if regression_results:
            add_regression_overlay(
                ax=ax1,
                strategy=strategy,
                x_values=cur_x_pos,
                regression_results=regression_results,
            )

        if has_any_error:
            if all(err is not None and not isinstance(err, (list, tuple)) for err in y_errors):
                ax1.errorbar(
                    cur_x_pos,
                    y_values,
                    yerr=y_errors,
                    fmt=f"-{marker}",
                    label=strategy,
                    color=color,
                    capsize=4,
                )
            else:
                ax1.plot(
                    cur_x_pos,
                    y_values,
                    marker=marker,
                    label=strategy,
                    color=color,
                )
        else:
            ax1.plot(
                cur_x_pos,
                y_values,
                marker=marker,
                label=strategy,
                color=color,
            )

    ax1.set_xlabel(xlabel)
    ax1.set_xticks(x_pos)
    ax1.set_xticklabels(x_tick_labels, rotation=45, ha="right", fontsize=8)
    ax1.set_ylabel(ylabel, fontsize=14)

    if title is not None:
        ax1.set_title(title, fontsize=16)

    ax1.grid(True, alpha=0.3)
    ax1.legend()
    fig.tight_layout()

    if output_file is not None:
        output_file = analysis_path / Path(output_file)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        fig.savefig(output_file, dpi=200, bbox_inches="tight")
        plt.close(fig)
    else:
        plt.show()
