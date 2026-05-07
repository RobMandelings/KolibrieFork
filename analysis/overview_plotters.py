from pathlib import Path

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