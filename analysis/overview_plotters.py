from pathlib import Path

from constants import STRATEGIES
from overview_plotting import make_overview_plotter
from series import perc_overlap_label

plot_mean_throughput_overlap = make_overview_plotter(
    y_col="thr_mean",
    title="Mean throughput",
    xlabel="% Overlap",
    ylabel="Throughput (events/s)",
    workload_index_col="window.slide",
    descending=False,
    output_file=Path("overviews") / "throughput" / "mean" / "mean_throughput.png",
    label_fn=perc_overlap_label,
)


def make_default_overview_plotters():
    plotters = []

    metrics = [
        {
            "y_col": "thr_mean",
            "yerr_col": "thr_std_dev",  # Toggle to show standard deviation as error bars
            "title": "Mean throughput",
            "subdir": Path("overviews") / "throughput" / "mean",
            "filename": "mean_throughput",
        },
        {
            "y_col": "thr_median",
            "title": "Median throughput",
            "subdir": Path("overviews") / "throughput" / "median",
            "filename": "median_throughput",
        },
    ]

    for metric in metrics:
        y_col = metric["y_col"]
        yerr_col = metric.get("yerr_col")
        title = metric["title"]
        subdir = metric["subdir"]
        filename = metric["filename"]

        # Plot each strategy individually
        for strategy in [None] + STRATEGIES:
            if strategy is None:
                suffix = "all"
                strategies = None
            else:
                suffix = strategy
                strategies = [strategy]

            plotters.append(
                make_overview_plotter(
                    y_col=y_col,
                    title=f"{title} ({strategy})",
                    xlabel="% Overlap",
                    ylabel="Throughput (events/s)",
                    workload_index_col="window.slide",
                    descending=False,
                    output_file=subdir / f"{filename}_{suffix}.png",
                    label_fn=perc_overlap_label,
                    strategies=strategies,
                    yerr_col=yerr_col
                )
            )

    return plotters
