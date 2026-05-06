from pathlib import Path
from typing import Any

from constants import STRATEGIES
from series import plot_strategy_series, PlotterConfig, build_strategy_series
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
        config=build_relative_median_throughput_config(workloads, "Median throughput", "% Overlap", "Throughput (events/s)",
                                                       perc_overlap_getter)
    )

    plot_overviews(
        workloads=workloads,
        analysis_path=analysis_path,
        config=build_relative_mean_throughput_config(workloads, "Median throughput", "% Overlap", "Throughput (events/s)",
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
