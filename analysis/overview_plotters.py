from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Callable, Any

import pandas as pd

from constants import STRATEGIES
from overview_plotting import make_overview_plotter
from series import perc_overlap_label, slide_label


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


def identity_preprocess(df: pd.DataFrame) -> pd.DataFrame:
    return df


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


def make_default_overview_plotters(
        x_variant: PlotVariant,
        y_config: YConfig,
        descending: bool = False,
        title: str | None = None,
):

    plotters = []

    base_title = title or y_config.default_title

    for strategy in [None] + STRATEGIES:
        if strategy is None:
            suffix = "all"
            strategies = None
            strategy_title = "all"
        else:
            suffix = strategy
            strategies = [strategy]
            strategy_title = strategy

        plotters.append(
            make_overview_plotter(
                y_col=y_config.y_col,
                yerr_col=y_config.yerr_col,
                title=f"{base_title} ({strategy_title})",
                xlabel=x_variant.x_label,
                ylabel=y_config.ylabel,
                workload_index_col=x_variant.workload_index_col,
                descending=descending,
                output_file=y_config.subdir / f"{y_config.filename}_{suffix}.png",
                label_fn=x_variant.label_fn,
                strategies=strategies,
            )
        )

    return plotters