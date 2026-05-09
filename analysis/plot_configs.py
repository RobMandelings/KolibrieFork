from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Callable, Any

from series import slide_label, perc_overlap_label


class PlotVariant(Enum):
    NR_EVENTS = (
        "Nr events",
        lambda row: row.get("nr_events"),
        "nr_events",
    )
    SLIDE = (
        "Slide",
        slide_label,
        "window.slide",
    )
    PERC_OVERLAP = (
        "% Overlap",
        perc_overlap_label,
        "window.slide",
    )

    def __init__(
            self,
            x_label: str,
            label_fn: Callable[[Any], str],
            workload_index_col: str,
    ):
        self.x_label = x_label
        self.label_fn = label_fn
        self.workload_index_col = workload_index_col


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