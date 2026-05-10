from __future__ import annotations

from pathlib import Path
from typing import Any, Callable

import pandas as pd
from matplotlib import pyplot as plt

from constants import STRATEGY_COLORS, STRATEGY_MARKERS
from plot_configs import PlotVariant, YConfig
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


def make_strategy_windows_overview_plotter(
        *,
        x_variant: PlotVariant,
        y_config: YConfig,
        strategy: str,
        windows: list[int] | None = None,
        descending: bool = False,
        title: str | None = None,
        nr_windows_col: str = "nr_windows",
        x_label_col: str = "x_label",
        strategy_col: str = "strategy",
):
    suffix = f"{strategy}_windows"

    def plotter(df: pd.DataFrame, analysis_path):
        output_file = y_config.subdir / f"{y_config.filename}_{suffix}.png"
        plot_title = title or f"{y_config.default_title} ({strategy} windows)"

        print(f"Plotting to output: {output_file}")
        plot_single_strategy_windows_overview_from_df(
            df=df,
            analysis_path=analysis_path,
            label_fn=x_variant.label_fn,
            y_col=y_config.y_col,
            xlabel=x_variant.x_label,
            ylabel=y_config.ylabel,
            workload_index_col=x_variant.workload_index_col,
            strategy=strategy,
            nr_windows_col=nr_windows_col,
            windows=windows,
            title=plot_title,
            output_file=output_file,
            x_label_col=x_label_col,
            strategy_col=strategy_col,
            yerr_col=y_config.yerr_col,
            descending=descending,
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

WINDOW_LINESTYLES = {
    1: "-",
    2: "--",
    5: "-.",
    10: ":",
}

WINDOW_MARKER_FACE = {
    1: None,
    2: None,
    5: "none",
    10: "none",
}


def _resolve_workload_order(df: pd.DataFrame, workload_index_col, descending: bool):
    return (
        df[workload_index_col]
            .drop_duplicates()
            .sort_values(ascending=not descending)
            .tolist()
    )


def _build_x_axis_metadata(
        df: pd.DataFrame,
        workload_order,
        workload_index_col,
        x_label_col: str,
):
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

    return x_pos, workload_index_to_x, x_tick_labels


def _compute_regression_results(
        df: pd.DataFrame,
        y_col: str,
        strategy_col: str,
        descending: bool,
):
    if not OVERLAY_REGRESSION:
        return None

    return linear_trend_per_strategy_df(
        df=df,
        y_col=y_col,
        strategy_col=strategy_col,
        descending=descending,
        x_order_col="window.slide",
    )


def _collect_series_plot_data(
        series_df: pd.DataFrame,
        workload_order,
        workload_index_col,
        workload_index_to_x: dict,
        y_col: str,
        yerr_col=None,
):
    cur_x_pos = []
    y_values = []
    y_errors = []
    has_any_error = False

    for workload_idx in workload_order:
        row_df = series_df[series_df[workload_index_col] == workload_idx]
        if row_df.empty:
            continue

        for _, row in row_df.iterrows():
            cur_x_pos.append(workload_index_to_x[workload_idx])
            y_values.append(row[y_col])

            if yerr_col is not None and yerr_col in row_df.columns and pd.notna(row[yerr_col]):
                y_errors.append(row[yerr_col])
                has_any_error = True
            else:
                y_errors.append(None)

    return cur_x_pos, y_values, y_errors, has_any_error


def _plot_series(
        ax,
        *,
        label: str,
        cur_x_pos,
        y_values,
        y_errors,
        has_any_error: bool,
        color=None,
        marker="o",
        linestyle="-",
        markerfacecolor=None,
):
    if not cur_x_pos:
        return

    plot_kwargs = {
        "label": label,
        "color": color,
        "marker": marker,
        "linestyle": linestyle,
    }

    if markerfacecolor is not None:
        plot_kwargs["markerfacecolor"] = markerfacecolor

    if has_any_error and all(err is not None and not isinstance(err, (list, tuple)) for err in y_errors):
        ax.errorbar(
            cur_x_pos,
            y_values,
            yerr=y_errors,
            capsize=4,
            **plot_kwargs,
        )
    else:
        ax.plot(
            cur_x_pos,
            y_values,
            **plot_kwargs,
        )


def _style_and_finalize_plot(
        fig,
        ax,
        *,
        x_pos,
        x_tick_labels,
        xlabel: str,
        ylabel: str,
        title=None,
        analysis_path: Path,
        output_file=None,
):
    ax.set_xlabel(xlabel)
    ax.set_xticks(x_pos)
    ax.set_xticklabels(x_tick_labels, rotation=45, ha="right", fontsize=8)
    ax.set_ylabel(ylabel, fontsize=14)

    if title is not None:
        ax.set_title(title, fontsize=16)

    ax.grid(True, alpha=0.3)
    ax.legend()
    fig.tight_layout()

    if output_file is not None:
        output_file = analysis_path / Path(output_file)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        fig.savefig(output_file, dpi=200, bbox_inches="tight")
        plt.close(fig)
    else:
        plt.show()


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

    workload_order = _resolve_workload_order(df, workload_index_col, descending)
    x_pos, workload_index_to_x, x_tick_labels = _build_x_axis_metadata(
        df=df,
        workload_order=workload_order,
        workload_index_col=workload_index_col,
        x_label_col=x_label_col,
    )

    fig, ax1 = plt.subplots(figsize=(16, 6))

    regression_results = _compute_regression_results(
        df=df,
        y_col=y_col,
        strategy_col=strategy_col,
        descending=descending,
    )

    for strategy in strategies:
        strat_df = df[df[strategy_col] == strategy].copy()
        if strat_df.empty:
            continue

        strat_df = strat_df[strat_df[workload_index_col].isin(workload_order)]
        strat_df = strat_df.sort_values(by=workload_index_col)

        cur_x_pos, y_values, y_errors, has_any_error = _collect_series_plot_data(
            series_df=strat_df,
            workload_order=workload_order,
            workload_index_col=workload_index_col,
            workload_index_to_x=workload_index_to_x,
            y_col=y_col,
            yerr_col=yerr_col,
        )

        marker = STRATEGY_MARKERS.get(strategy, "o")
        color = STRATEGY_COLORS.get(strategy, None)

        if regression_results:
            add_regression_overlay(
                ax=ax1,
                strategy=strategy,
                x_values=cur_x_pos,
                regression_results=regression_results,
            )

        _plot_series(
            ax1,
            label=strategy,
            cur_x_pos=cur_x_pos,
            y_values=y_values,
            y_errors=y_errors,
            has_any_error=has_any_error,
            color=color,
            marker=marker,
            linestyle="-",
        )

    _style_and_finalize_plot(
        fig,
        ax1,
        x_pos=x_pos,
        x_tick_labels=x_tick_labels,
        xlabel=xlabel,
        ylabel=ylabel,
        title=title,
        analysis_path=analysis_path,
        output_file=output_file,
    )


def plot_single_strategy_windows_overview_from_df(
        df,
        analysis_path: Path,
        label_fn: Callable[[pd.Series], Any],
        y_col: str,
        xlabel: str,
        ylabel: str,
        workload_index_col,
        strategy: str,
        nr_windows_col="nr_windows",
        windows=None,
        title=None,
        output_file=None,
        x_label_col="x_label",
        strategy_col="strategy",
        yerr_col=None,
        descending: bool = False,
):
    df = add_label_column(df, label_fn=label_fn, out_col=x_label_col)
    df = df.copy()

    df = df[df[strategy_col] == strategy].copy()
    if df.empty:
        print(f"WARN: no rows found for strategy '{strategy}'.")
        return

    available_windows = list(df[nr_windows_col].dropna().unique())
    available_windows = sorted(available_windows)

    if windows is None:
        windows = available_windows
    else:
        windows = [w for w in windows if w in available_windows]

    if len(windows) > 4:
        print(f"WARN: received {len(windows)} windows, only plotting first 4.")
        windows = windows[:4]

    if not windows:
        print(f"WARN: no windows to plot for strategy '{strategy}'.")
        return

    workload_order = _resolve_workload_order(df, workload_index_col, descending)
    x_pos, workload_index_to_x, x_tick_labels = _build_x_axis_metadata(
        df=df,
        workload_order=workload_order,
        workload_index_col=workload_index_col,
        x_label_col=x_label_col,
    )

    fig, ax1 = plt.subplots(figsize=(16, 6))

    base_color = STRATEGY_COLORS.get(strategy, None)
    base_marker = STRATEGY_MARKERS.get(strategy, "o")

    for nr_windows in windows:
        window_df = df[df[nr_windows_col] == nr_windows].copy()
        if window_df.empty:
            print(
                f"WARN: no rows found for strategy '{strategy}' "
                f"and {nr_windows_col}={nr_windows}."
            )
            continue

        window_df = window_df[window_df[workload_index_col].isin(workload_order)]
        window_df = window_df.sort_values(by=workload_index_col)

        cur_x_pos, y_values, y_errors, has_any_error = _collect_series_plot_data(
            series_df=window_df,
            workload_order=workload_order,
            workload_index_col=workload_index_col,
            workload_index_to_x=workload_index_to_x,
            y_col=y_col,
            yerr_col=yerr_col,
        )

        linestyle = WINDOW_LINESTYLES.get(nr_windows, "-")
        markerfacecolor = WINDOW_MARKER_FACE.get(nr_windows, None)

        _plot_series(
            ax1,
            label=f"{strategy} (windows={nr_windows})",
            cur_x_pos=cur_x_pos,
            y_values=y_values,
            y_errors=y_errors,
            has_any_error=has_any_error,
            color=base_color,
            marker=base_marker,
            linestyle=linestyle,
            markerfacecolor=markerfacecolor,
        )

    if title is None:
        title = f"{strategy} across window counts"

    _style_and_finalize_plot(
        fig,
        ax1,
        x_pos=x_pos,
        x_tick_labels=x_tick_labels,
        xlabel=xlabel,
        ylabel=ylabel,
        title=title,
        analysis_path=analysis_path,
        output_file=output_file,
    )

# def plot_overview_from_df(
#         df,
#         analysis_path: Path,
#         label_fn: Callable[[pd.Series], Any],
#         y_col: str,
#         xlabel: str,
#         ylabel: str,
#         workload_index_col,
#         strategies=None,
#         title=None,
#         output_file=None,
#         x_label_col="x_label",
#         strategy_col="strategy",
#         yerr_col=None,
#         descending: bool = False,
# ):
#     df = add_label_column(df, label_fn=label_fn, out_col=x_label_col)
#     df = df.copy()
#
#     if strategies is None:
#         strategies = list(df[strategy_col].dropna().unique())
#
#     workload_order = (
#         df[workload_index_col]
#             .drop_duplicates()
#             .sort_values(ascending=not descending)
#             .tolist()
#     )
#
#     x_pos = list(range(len(workload_order)))
#     workload_index_to_x = {w: i for i, w in enumerate(workload_order)}
#
#     x_tick_labels = []
#     for workload_idx in workload_order:
#         subset = df[df[workload_index_col] == workload_idx]
#         if subset.empty:
#             x_tick_labels.append(str(workload_idx))
#         elif x_label_col in subset.columns:
#             x_tick_labels.append(str(subset.iloc[0][x_label_col]))
#         else:
#             x_tick_labels.append(str(workload_idx))
#
#     fig, ax1 = plt.subplots(figsize=(16, 6))
#
#     if OVERLAY_REGRESSION:
#         regression_results = linear_trend_per_strategy_df(
#             df=df,
#             y_col=y_col,
#             strategy_col=strategy_col,
#             descending=descending,
#             x_order_col="window.slide",
#         )
#     else:
#         regression_results = None
#
#     for strategy in strategies:
#         strat_df = df[df[strategy_col] == strategy].copy()
#         if strat_df.empty:
#             continue
#
#         strat_df = strat_df[strat_df[workload_index_col].isin(workload_order)]
#         strat_df = strat_df.sort_values(by=workload_index_col)
#
#         cur_x_pos = []
#         y_values = []
#         y_errors = []
#         has_any_error = False
#
#         for workload_idx in workload_order:
#             row_df = strat_df[strat_df[workload_index_col] == workload_idx]
#             if row_df.empty:
#                 continue
#
#             for _, row in row_df.iterrows():
#                 cur_x_pos.append(workload_index_to_x[workload_idx])
#                 y_values.append(row[y_col])
#
#                 if yerr_col is not None and yerr_col in row_df.columns and pd.notna(row[yerr_col]):
#                     y_errors.append(row[yerr_col])
#                     has_any_error = True
#                 else:
#                     y_errors.append(None)
#
#         if not cur_x_pos:
#             continue
#
#         marker = STRATEGY_MARKERS.get(strategy, "o")
#         color = STRATEGY_COLORS.get(strategy, None)
#
#         if regression_results:
#             add_regression_overlay(
#                 ax=ax1,
#                 strategy=strategy,
#                 x_values=cur_x_pos,
#                 regression_results=regression_results,
#             )
#
#         if has_any_error:
#             if all(err is not None and not isinstance(err, (list, tuple)) for err in y_errors):
#                 ax1.errorbar(
#                     cur_x_pos,
#                     y_values,
#                     yerr=y_errors,
#                     fmt=f"-{marker}",
#                     label=strategy,
#                     color=color,
#                     capsize=4,
#                 )
#             else:
#                 ax1.plot(
#                     cur_x_pos,
#                     y_values,
#                     marker=marker,
#                     label=strategy,
#                     color=color,
#                 )
#         else:
#             ax1.plot(
#                 cur_x_pos,
#                 y_values,
#                 marker=marker,
#                 label=strategy,
#                 color=color,
#             )
#
#     ax1.set_xlabel(xlabel)
#     ax1.set_xticks(x_pos)
#     ax1.set_xticklabels(x_tick_labels, rotation=45, ha="right", fontsize=8)
#     ax1.set_ylabel(ylabel, fontsize=14)
#
#     if title is not None:
#         ax1.set_title(title, fontsize=16)
#
#     ax1.grid(True, alpha=0.3)
#     ax1.legend()
#     fig.tight_layout()
#
#     if output_file is not None:
#         output_file = analysis_path / Path(output_file)
#         output_file.parent.mkdir(parents=True, exist_ok=True)
#         fig.savefig(output_file, dpi=200, bbox_inches="tight")
#         plt.close(fig)
#     else:
#         plt.show()
