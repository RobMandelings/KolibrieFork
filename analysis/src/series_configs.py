from pathlib import Path

import error_getter
from my_stats import thr_mean, thr_median
from series import PlotterConfig


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


def get_total_bytes_window_closed(workloads, workload_key, strat_key):
    total_bytes_window_closed = workloads[workload_key]["strategies"][strat_key]["memory"]["total_bytes"].get(
        ["vector_clone_from_window_closed"])
    # Will be non for legacy (this function does not exist there)
    if total_bytes_window_closed is None:
        return 0
    return total_bytes_window_closed


def get_total_bytes(workloads, workload_key, strat_key):
    total_bytes = workloads[workload_key]["strategies"][strat_key]["memory"]["total_bytes"]["TOTAL"]
    return total_bytes


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


def build_total_bytes_config(title, x_label, y_label, x_label_getter, subdir):
    return PlotterConfig(
        title=title,
        value_getter=get_total_bytes,
        error_getter=None,
        x_label=x_label,
        y_label=y_label,
        subdir=subdir,
        x_label_getter=x_label_getter,
    )


def build_median_throughput_config(title, x_label, y_label, x_label_getter):
    return PlotterConfig(
        title=title,
        value_getter=get_thr_med,
        error_getter=None,
        x_label=x_label,
        y_label=y_label,
        subdir=Path("throughput") / "median",
        x_label_getter=x_label_getter,
    )


def build_mean_throughput_config(title, x_label, y_label, x_label_getter):
    return PlotterConfig(
        title=title,
        value_getter=get_thr_mean,
        error_getter=error_getter.get_throughput_conf_int_error,
        x_label=x_label,
        y_label=y_label,
        subdir=Path("throughput") / "mean",
        x_label_getter=x_label_getter,
    )


def build_relative_median_throughput_config(
        workloads,
        title,
        x_label,
        y_label,
        x_label_getter,
):
    return PlotterConfig(
        title=title,
        value_getter=make_relative_to_baseline_getter(workloads, get_thr_med),
        error_getter=None,
        x_label=x_label,
        y_label=y_label,
        subdir=Path("throughput_relative") / "median",
        x_label_getter=x_label_getter,
    )


def build_relative_mean_throughput_config(
        workloads,
        title,
        x_label,
        y_label,
        x_label_getter,
):
    return PlotterConfig(
        title=title,
        value_getter=make_relative_to_baseline_getter(workloads, get_thr_mean),
        error_getter=error_getter.get_throughput_conf_int_error,
        x_label=x_label,
        y_label=y_label,
        subdir=Path("throughput_relative") / "mean",
        x_label_getter=x_label_getter,
    )
