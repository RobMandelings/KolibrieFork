from pathlib import Path
from typing import Any

from constants import STRATEGIES, ESTIMATES
import error_getter
from my_stats import thr_mean, thr_median
from series import plot_strategy_series, SeriesBuildConfig


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
    return f"{perc_overlap:.2%}"
    # return f"{perc_overlap:.2%} (size={size}, slide={slide})"


def get_thr_mean(strat_data, workload_key, workload_data):
    return thr_mean(strat_data["throughput"]["sample"])


def get_thr_med(strat_data, workload_key, workload_data):
    return thr_median(strat_data["throughput"]["sample"])


def get_thr_mean_no_outlier(strat_data, workload_key, workload_data):
    return thr_mean(strat_data["throughput"]["sample_no_outlier"])


def get_thr_med_no_outlier(strat_data, workload_key, workload_data):
    return thr_median(strat_data["throughput"]["sample_no_outlier"])


def generate_throughput_overview_plots(
        workloads: Any,
        analysis_path: Path,
        remove_outliers=False
) -> None:
    """
    Generate throughput overview plots for all strategies and estimates.

    Parameters
    ----------
    workloads : ...
        Workload data structure consumed by plot_throughput_from_workloads.
    analysis_path : Path
        Base path where the 'overviews/png/throughput/...' tree will be written.
    """
    for strat in STRATEGIES + [None]:
        for key, title in ESTIMATES.items():
            if "median" in key:
                err_get_fn = None

                if remove_outliers:
                    val_get_fn = get_thr_med_no_outlier
                else:
                    val_get_fn = get_thr_med
            else:
                err_get_fn = error_getter.get_throughput_conf_int_error

                if remove_outliers:
                    val_get_fn = get_thr_mean_no_outlier
                else:
                    val_get_fn = get_thr_mean

            thr_mean_config = SeriesBuildConfig(
                value_getter=val_get_fn,
                error_getter=err_get_fn,
                x_label_getter=perc_overlap_getter,
            )

            if strat is not None:
                strategies = [strat]
            else:
                strategies = None

            filename = strat if strat is not None else "all"
            if remove_outliers:
                filename += "_no_outlier"

            plot_strategy_series(
                workloads=workloads,
                config=thr_mean_config,
                xlabel="slide",
                ylabel="throughput (events/s)",
                strategies=strategies,
                title=title,
                output_file=(
                        analysis_path
                        / "overviews"
                        / "png"
                        / "throughput"
                        / key
                        / filename
                ),
            )
