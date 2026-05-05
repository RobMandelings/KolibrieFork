from pathlib import Path
from typing import Any

from constants import STRATEGIES, ESTIMATES
import error_getter
from my_stats import thr_mean, thr_median
from series import plot_strategy_series, SeriesBuildConfig, build_strategy_series
from series_export import export_result_to_csv


def slide_x_label_getter(strat_data, workload_key, workload_data):
    workload = workload_data.get("workload")
    slide = workload["window"]["slide"]
    return f"{slide}"


def perc_overlap_getter(strat_data, workload_key, workload_data):
    workload = workload_data.get("workload", {})
    window = workload.get("window", {})
    size = window.get("size")
    slide = window.get("slide")

    if size in (None, 0) or slide is None:
        return "N/A"

    perc_overlap = (size - slide) / size
    if perc_overlap < 0:
        perc_overlap = 0

    return f"{perc_overlap:.2%}"


def get_thr_mean(workloads, workload_key, strat_key):
    strat_data = workloads[workload_key]["strategies"][strat_key]
    return thr_mean(strat_data["throughput"]["sample"])


def get_thr_med(workloads, workload_key, strat_key):
    strat_data = workloads[workload_key]["strategies"][strat_key]
    return thr_median(strat_data["throughput"]["sample"])


def get_thr_mean_no_outlier(workloads, workload_key, strat_key):
    strat_data = workloads[workload_key]["strategies"][strat_key]
    return thr_mean(strat_data["throughput"]["sample_no_outlier"])


def get_thr_med_no_outlier(workloads, workload_key, strat_key):
    strat_data = workloads[workload_key]["strategies"][strat_key]
    return thr_median(strat_data["throughput"]["sample_no_outlier"])


def make_relative_to_baseline_getter(workloads, value_getter):
    first_workload_key = next(iter(workloads))
    baseline_cache = {}

    def get_relative(workloads, workload_key, strat_key):
        if strat_key not in baseline_cache:
            baseline_cache[strat_key] = value_getter(workloads, first_workload_key, strat_key)

        baseline = baseline_cache[strat_key]
        value = value_getter(workloads, workload_key, strat_key)
        return value / baseline

    return get_relative


def build_throughput_series_config(workloads, key, relative=False):
    if "median" in key:
        base_value_getter = get_thr_med
        err_get_fn = None
    else:
        base_value_getter = get_thr_mean
        err_get_fn = error_getter.get_throughput_conf_int_error

    value_getter = (
        make_relative_to_baseline_getter(workloads, base_value_getter)
        if relative
        else base_value_getter
    )

    return SeriesBuildConfig(
        workloads=workloads,
        value_getter=value_getter,
        error_getter=err_get_fn,
        x_label_getter=perc_overlap_getter,
    )


def plot_throughput_overviews(
        workloads,
        analysis_path,
        estimates,
        strategies,
        relative=False,
):
    ylabel = "relative throughput" if relative else "throughput (events/s)"
    subdir = "throughput_relative" if relative else "throughput"

    print("Plotting throughput overviews")

    configs = {}
    series_dict = {}

    print(f"Building strategy series")
    for key in estimates.keys():
        configs[key] = build_throughput_series_config(
            workloads=workloads,
            key=key,
            relative=relative,
        )
        series_dict[key] = build_strategy_series(configs[key])

        print("Exporting to CSV")
        export_result_to_csv(series_dict[key], csv_path=(
                analysis_path
                / "overviews"
                / subdir
                / key
                / f"all.csv"
        ))

    for strat in strategies + [None]:
        for key, title in estimates.items():

            selected_strategies = [strat] if strat is not None else None
            filename = strat if strat is not None else "all"

            print("Exporting to PNG")
            plot_strategy_series(
                workloads=workloads,
                series=series_dict[key],
                xlabel="slide",
                ylabel=ylabel,
                strategies=selected_strategies,
                title=title,
                output_file=(
                        analysis_path
                        / "overviews"
                        / subdir
                        / key
                        / filename
                ),
            )


def generate_throughput_overview_plots(
        workloads: Any,
        analysis_path: Path,
) -> None:
    plot_throughput_overviews(
        workloads=workloads,
        analysis_path=analysis_path,
        estimates=ESTIMATES,
        strategies=STRATEGIES,
        relative=False,
    )

    plot_throughput_overviews(
        workloads=workloads,
        analysis_path=analysis_path,
        estimates=ESTIMATES,
        strategies=STRATEGIES,
        relative=True,
    )
