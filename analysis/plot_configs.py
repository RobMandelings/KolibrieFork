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


def x_size(descending: bool = False) -> XConfig:
    return XConfig(
        x_label="Size",
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


Y_THR_MEAN = YConfig(
    y_col="thr_mean",
    yerr_col="thr_std_dev",
    default_title="Mean throughput",
    ylabel="Throughput (events/s)",
    subdir=Path("overviews") / "throughput" / "mean",
    filename="mean_throughput",
)

Y_THR_MEDIAN = YConfig(
    y_col="thr_median",
    yerr_col=None,
    default_title="Median throughput",
    ylabel="Throughput (events/s)",
    subdir=Path("overviews") / "throughput" / "median",
    filename="median_throughput",
)

Y_MEM = YConfig(
    y_col="mem_total",
    yerr_col=None,
    default_title="Memory usage",
    ylabel="Bytes",
    subdir=Path("overviews") / "memory",
    filename="total_bytes",
)

Y_THR_MEAN_REL = YConfig(
    y_col="thr_mean_rel",
    yerr_col=None,
    default_title="Mean throughput (relative)",
    ylabel="Factor",
    subdir=Path("overviews") / "throughput" / "relative" / "mean",
    filename="mean_throughput",
)

Y_THR_MEDIAN_REL = YConfig(
    y_col="thr_median_rel",
    yerr_col=None,
    default_title="Median throughput",
    ylabel="Factor",
    subdir=Path("overviews") / "throughput" / "relative" / "median",
    filename="median_throughput",
)

Y_THR_MEAN_REL_WINDOW = YConfig(
    y_col="thr_mean_rel_window",
    yerr_col=None,
    default_title="Mean throughput (relative to windows=1)",
    ylabel="Factor",
    subdir=Path("overviews") / "throughput" / "relative_window" / "mean",
    filename="mean_throughput_rel_window",
)

Y_THR_MEDIAN_REL_WINDOW = YConfig(
    y_col="thr_mean_rel_window",
    yerr_col=None,
    default_title="Median throughput (relative to windows=1)",
    ylabel="Factor",
    subdir=Path("overviews") / "throughput" / "relative_window" / "median",
    filename="median_throughput_rel_window",
)
