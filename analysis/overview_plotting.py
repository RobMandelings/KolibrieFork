from pathlib import Path
from typing import Any

import pandas as pd
from matplotlib import pyplot as plt

from constants import STRATEGIES, STRATEGY_COLORS, STRATEGY_MARKERS
from series import plot_strategy_series, PlotterConfig, build_strategy_series, workloads_to_dataframe, \
    linear_trend_per_strategy_df, add_regression_overlay
from series_configs import build_median_throughput_config, build_mean_throughput_config, perc_overlap_getter, \
    build_total_bytes_config, get_total_bytes, get_total_bytes_window_closed, build_relative_median_throughput_config, \
    build_relative_mean_throughput_config
from series_export import export_result_to_csv


# def build_throughput_series_config(workloads, key, relative=False):
#     if "median" in key:
#         base_value_getter = get_thr_med
#         err_get_fn = None
#     else:
#         base_value_getter = get_thr_mean
#         err_get_fn = error_getter.get_throughput_conf_int_error
#
#     value_getter = (
#         make_relative_to_baseline_getter(workloads, base_value_getter)
#         if relative
#         else base_value_getter
#     )
#
#     return PlotterConfig(
#         value_getter=value_getter,
#         error_getter=err_get_fn,
#         x_label_getter=perc_overlap_getter,
#     )


def plot_overviews(
        workloads,
        analysis_path,
        config: PlotterConfig,
):
    print(f"Plotting overviews: {config.title}")

    series = build_strategy_series(config, workloads)

    print("Exporting to CSV")
    export_result_to_csv(
        series,
        csv_path=(
                analysis_path
                / "overviews"
                / config.subdir
                / "all.csv"
        ),
    )

    for strat in STRATEGIES + [None]:
        selected_strategies = [strat] if strat is not None else None
        filename = strat if strat is not None else "all"

        print(f"Exporting plot to PNG for strat: {strat}")
        plot_strategy_series(
            workloads=workloads,
            series=series,
            xlabel=config.x_label,
            ylabel=config.y_label,
            strategies=selected_strategies,
            title=config.title,
            output_file=(
                    analysis_path
                    / "overviews"
                    / config.subdir
                    / filename
            ),
        )


def generate_throughput_overview_plots(
        workloads: Any,
        analysis_path: Path,
) -> None:
    plot_overviews(
        workloads=workloads,
        analysis_path=analysis_path,
        config=build_mean_throughput_config("Mean throughput", "% Overlap", "Throughput (events/s)",
                                            perc_overlap_getter)
    )

    plot_overviews(
        workloads=workloads,
        analysis_path=analysis_path,
        config=build_median_throughput_config("Median throughput", "% Overlap", "Throughput (events/s)",
                                              perc_overlap_getter)
    )

    plot_overviews(
        workloads=workloads,
        analysis_path=analysis_path,
        config=build_relative_median_throughput_config(workloads, "Median throughput", "% Overlap",
                                                       "Throughput (events/s)",
                                                       perc_overlap_getter)
    )

    plot_overviews(
        workloads=workloads,
        analysis_path=analysis_path,
        config=build_relative_mean_throughput_config(workloads, "Median throughput", "% Overlap",
                                                     "Throughput (events/s)",
                                                     perc_overlap_getter)
    )

    plot_overviews(
        workloads=workloads,
        analysis_path=analysis_path,
        config=build_total_bytes_config("Total bytes", "% Overlap", "Bytes", perc_overlap_getter, subdir=Path("memory"))
    )

    plot_overviews(
        workloads=workloads,
        analysis_path=analysis_path,
        config=PlotterConfig(
            title="Total bytes from window close",
            value_getter=get_total_bytes_window_closed,
            error_getter=None,
            x_label="% Overlap",
            y_label="Bytes",
            subdir=Path("memory") / "window_close",
            x_label_getter=perc_overlap_getter,
        ))

    # plot_throughput_overviews(
    #     workloads=workloads,
    #     analysis_path=analysis_path,
    #     estimates=ESTIMATES,
    #     strategies=STRATEGIES,
    #     relative=True,
    # )


def plot_strategy_series_df(
        df,
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

    regression_results = linear_trend_per_strategy_df(
        df=df,
        y_col=y_col,
        strategy_col=strategy_col,
        descending=descending,
        x_order_col="window.slide",
    )

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
        output_file = Path(output_file)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        fig.savefig(output_file, dpi=200, bbox_inches="tight")
        plt.close(fig)
    else:
        plt.show()
