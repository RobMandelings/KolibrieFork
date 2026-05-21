from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Any

from series import slide_label, perc_overlap_label, size_label


@dataclass(frozen=True)
class XConfig:
    x_label: str
    label_fn: Callable[[Any], str]
    workload_index_col: str
    descending: bool


def x_nr_events(descending: bool = False) -> XConfig:
    return XConfig(
        x_label="Nr events",
        label_fn=lambda row: row.get("nr_events"),
        workload_index_col="nr_events",
        descending=descending,
    )


def x_nr_windows(descending: bool = False) -> XConfig:
    return XConfig(
        x_label="Nr windows",
        label_fn=lambda row: row.get("nr_windows"),
        workload_index_col="nr_windows",
        descending=descending,
    )


def x_slide(descending: bool = False) -> XConfig:
    return XConfig(
        x_label="Slide",
        label_fn=slide_label,
        workload_index_col="window.slide",
        descending=descending,
    )


def x_nr_elems(descending: bool = False) -> XConfig:
    return XConfig(
        x_label="Number of elements",
        label_fn=size_label,
        workload_index_col="window.size",
        descending=descending,
    )


def x_perc_overlap(descending: bool = False) -> XConfig:
    return XConfig(
        x_label="% Overlap",
        label_fn=perc_overlap_label,
        workload_index_col="window.slide",
        descending=descending,
    )


@dataclass(frozen=True)
class YConfig:
    y_col: str
    yerr_col: str | None
    default_title: str
    ylabel: str
    subdir: Path
    filename: str
    y_log: bool = False
    y_log_base: float = 10.0


def with_log_filename(filename: str, y_log: bool) -> str:
    return f"{filename}_log" if y_log else filename


def y_lat_mean(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="ns_mean",
        yerr_col=None,
        default_title="Mean latency",
        ylabel="Nanoseconds",
        subdir=Path("overviews") / "nanoseconds" / "mean",
        filename=with_log_filename("mean_ns", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_lat_mean_rel(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="ns_mean_rel",
        yerr_col=None,
        default_title="Mean latency (relative)",
        ylabel="Factor",
        subdir=Path("overviews") / "latency" / "relative" / "mean",
        filename=with_log_filename("mean_latency_rel", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_lat_rel_to_slice(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="ns_mean_rel_slice",
        yerr_col=None,
        default_title="Relative latency (compared to slice)",
        ylabel="Nanoseconds",
        subdir=Path("overviews") / "nanoseconds" / "mean",
        filename=with_log_filename("latency_slice_rel", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_lat_mean_diff(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="ns_mean_diff_slice",
        yerr_col=None,
        default_title="Relative latency (compared to slice, abs difference)",
        ylabel="Nanoseconds",
        subdir=Path("overviews") / "nanoseconds" / "mean",
        filename=with_log_filename("latency_slice_diff", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_thr_rel_to_slice(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="thr_mean_rel_slice",
        yerr_col=None,
        default_title="Throughput (relative to slice)",
        ylabel="Relative throughput",
        subdir=Path("overviews") / "throughput" / "relative",
        filename=with_log_filename("thr_rel_to_slice", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_thr_mean(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="thr_mean",
        yerr_col="thr_std_dev",
        default_title="Mean throughput",
        ylabel="Throughput (events/s)",
        subdir=Path("overviews") / "throughput" / "mean",
        filename=with_log_filename("mean_throughput", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_thr_median(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="thr_median",
        yerr_col=None,
        default_title="Median throughput",
        ylabel="Throughput (events/s)",
        subdir=Path("overviews") / "throughput" / "median",
        filename=with_log_filename("median_throughput", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_thr_mean_rel_prev(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="thr_mean_rel_prev",
        yerr_col=None,
        default_title="Mean throughput change relative to previous point",
        ylabel="Relative change",
        subdir=Path("overviews") / "throughput" / "relative_previous" / "mean",
        filename=with_log_filename("mean_throughput_rel_prev", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_thr_mean_rel(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="thr_mean_rel",
        yerr_col=None,
        default_title="Mean throughput (relative)",
        ylabel="Factor",
        subdir=Path("overviews") / "throughput" / "relative" / "mean",
        filename=with_log_filename("mean_throughput_rel", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_thr_median_rel(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="thr_median_rel",
        yerr_col=None,
        default_title="Median throughput",
        ylabel="Factor",
        subdir=Path("overviews") / "throughput" / "relative" / "median",
        filename=with_log_filename("median_throughput", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_mem(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="mem_total",
        yerr_col=None,
        default_title="Memory usage",
        ylabel="Bytes",
        subdir=Path("overviews") / "memory",
        filename=with_log_filename("total_bytes", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_mem_rel(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="mem_total_rel",
        yerr_col=None,
        default_title="Memory usage",
        ylabel="Bytes",
        subdir=Path("overviews") / "memory" / "relative",
        filename=with_log_filename("total_bytes_rel", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_thr_mean_rel_window(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="thr_mean_rel_window",
        yerr_col=None,
        default_title="Mean throughput (relative to windows=1)",
        ylabel="Factor",
        subdir=Path("overviews") / "throughput" / "relative_window" / "mean",
        filename=with_log_filename("mean_throughput_rel_window", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )


def y_thr_median_rel_window(y_log: bool = False, y_log_base: float = 10.0) -> YConfig:
    return YConfig(
        y_col="thr_mean_rel_window",
        yerr_col=None,
        default_title="Median throughput (relative to windows=1)",
        ylabel="Factor",
        subdir=Path("overviews") / "throughput" / "relative_window" / "median",
        filename=with_log_filename("median_throughput_rel_window", y_log),
        y_log=y_log,
        y_log_base=y_log_base,
    )
