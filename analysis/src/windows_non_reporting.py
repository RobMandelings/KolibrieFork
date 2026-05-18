from pathlib import Path

import pandas as pd

import overview_plotters
import plot_configs
from generate_plots import add_throughputs, add_relative_metric
from regression import report_strategy_regressions


def plot_relative_throughputs(df: pd.DataFrame, x_config: plot_configs.XConfig, folder_path):
    y_config = plot_configs.y_thr_mean_rel(y_log=False)
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


def plot_relative_latencies(df: pd.DataFrame, x_config: plot_configs.XConfig, folder_path):
    y_config = plot_configs.y_lat_mean_rel(False)

    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc", "arc", "legacy"]
    )
    plotter(df, folder_path)


def plot_mem_usage(df: pd.DataFrame, x_config: plot_configs.XConfig, folder_path):
    y_config = plot_configs.y_mem_rel(True)

    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc", "arc", "legacy"]
    )
    plotter(df, folder_path)


def windows_non_reporting():
    analysis_path = Path("../evaluation/windows_non_reporting/summary.csv")
    df = pd.read_csv(analysis_path)
    df = add_throughputs(df)
    folder_path = analysis_path.parent

    x_config = plot_configs.x_nr_windows(False)

    df = add_relative_metric(df,
                             strategy_col="strategy",
                             metric_col="ns_mean",
                             x_config=x_config
                             )

    df = add_relative_metric(df,
                             strategy_col="strategy",
                             metric_col="mem_total",
                             x_config=x_config
                             )

    print("RELATIVE REGRESSIONS TO SHOW DIFFERENCES BETWEEN STRATEGIES")
    report_strategy_regressions(df, x_config, plot_configs.y_lat_mean_rel(False), strategies=["clone", "slice", "rc", "arc", "legacy"])

    plot_relative_throughputs(df, x_config, folder_path)
    plot_relative_latencies(df, x_config, folder_path)
    plot_mem_usage(df, x_config, folder_path)


if __name__ == "__main__":
    windows_non_reporting()
