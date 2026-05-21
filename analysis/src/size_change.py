from pathlib import Path

import pandas as pd

import overview_plotters
import plot_configs
from generate_plots import add_throughputs, add_relative_to_slice

x_config = plot_configs.x_nr_elems(False)


def plot_mean_throughputs_log(
        df: pd.DataFrame,
        folder_path,
):
    y_config = plot_configs.y_thr_mean(y_log=True, y_log_base=10)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc"],
    )
    plotter(df, folder_path)


def plot_mean_throughputs(
        df: pd.DataFrame,
        folder_path,
):
    y_config = plot_configs.y_thr_mean()
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc"],
    )
    plotter(df, folder_path)


def plot_mean_latencies(
        df: pd.DataFrame,
        folder_path,
):
    y_config = plot_configs.y_lat_mean(y_log=False)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc"],
    )
    plotter(df, folder_path)


def plot_memory_usage(
        df: pd.DataFrame,
        folder_path,
):
    y_config = plot_configs.y_mem(y_log=False)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["slice", "rc", "clone"],
    )
    plotter(df, folder_path)


def plot_relative_throughputs(
        df: pd.DataFrame,
        folder_path,
):
    df = add_relative_to_slice(
        df,
        strategy_name="slice",
        metric_col="thr_mean",
    )
    y_config = plot_configs.y_thr_rel_to_slice(y_log=False)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice"],
    )
    plotter(df, folder_path)


def size_change(csv_path):
    analysis_path = Path(csv_path)
    df = pd.read_csv(analysis_path)
    df = add_throughputs(df)
    folder_path = analysis_path.parent

    plot_mean_throughputs(df, folder_path)
    plot_mean_throughputs_log(df, folder_path)
    plot_mean_latencies(df, folder_path)
    plot_relative_throughputs(df, folder_path)
    plot_memory_usage(df, folder_path)
