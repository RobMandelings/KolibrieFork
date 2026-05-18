from pathlib import Path

import pandas as pd

import plot_configs
from regression import report_strategy_regressions
from generate_plots import add_throughputs, decorate_df, add_relative_metric
import overview_plotters


def windows_non_reporting():
    analysis_path = Path("../evaluation/windows_non_reporting/summary.csv")
    df = pd.read_csv(analysis_path)
    df = add_throughputs(df)
    folder_path = analysis_path.parent

    x_config = plot_configs.x_nr_windows(False)
    y_config = plot_configs.Y_LAT_MEAN_REL

    df = add_relative_metric(df,
                             strategy_col="strategy",
                             metric_col="ns_mean",
                             x_config=x_config
                             )

    report_strategy_regressions(df, x_config, y_config, strategies=["clone", "slice", "rc", "arc", "legacy"])

    y_config = plot_configs.Y_THR_MEAN_REL
    df = add_relative_metric(df,
                             strategy_col="strategy",
                             metric_col="thr_mean",
                             x_config=x_config
                             )

    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc", "arc"]
    )
    plotter(df, folder_path)

    y_config = plot_configs.Y_LAT_MEAN_REL

    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc", "arc", "legacy"]
    )
    plotter(df, folder_path)


if __name__ == "__main__":
    windows_non_reporting()
