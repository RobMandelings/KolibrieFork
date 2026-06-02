from __future__ import annotations

from pathlib import Path
from typing import List

import pandas as pd

import overview_plotters
import plot_configs
from generate_plots import add_throughputs, add_relative_to_slice, add_relative_metric, add_relative_change_to_previous
from baseline_comparison_table import create_baseline_comparison_table, create_raw_value_table

x_config = plot_configs.x_nr_elems(False)


def plot_mean_throughputs_log(
        df: pd.DataFrame,
        folder_path,
):
    y_config = plot_configs.y_thr_mean(y_log=True, y_log_base=2)
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


def plot_relative_throughputs_to_slice(
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
        strategies=["clone", "slice", "rc"],
    )
    plotter(df, folder_path)


def plot_relative_throughputs(
        df: pd.DataFrame,
        folder_path,
):
    df = add_relative_metric(
        df,
        metric_col="thr_mean",
        x_config=x_config
    )
    y_config = plot_configs.y_thr_mean_rel(y_log=True)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc"],
    )
    plotter(df, folder_path)
    y_config = plot_configs.y_thr_mean_rel(y_log=False)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc"],
    )
    plotter(df, folder_path)


def plot_relative_throughputs_prev(
        df: pd.DataFrame,
        folder_path,
):
    df = add_relative_change_to_previous(
        df,
        metric_col="thr_mean",
        x_config=x_config,
        group_cols=["strategy"],
        relative_col="thr_mean_rel_prev",
    )

    y_config = plot_configs.y_thr_mean_rel_prev(y_log=False)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc"],
    )
    plotter(df, folder_path)


def export_default_baseline_comparison(
        df: pd.DataFrame,
        folder_path: Path,
        metric_col: str,
) -> pd.DataFrame:
    """
    Export the default baseline comparison table for the standard setup.
    """
    table = create_baseline_comparison_table(
        df=df,
        x_config=x_config,
        x_values=[1, 16, 256, 4096, 65536],
        strategies=["clone", "slice", "rc"],
        baseline_strategy="slice",
        metric_col=metric_col,
    )

    csv_path = folder_path / f"{metric_col}_baseline_comparison.csv"
    table.to_csv(csv_path)

    print(f"Comparison table exported to: {csv_path}")
    return table


def export_default_raw_table(
        df: pd.DataFrame,
        folder_path: Path,
        metric_col: str,
) -> pd.DataFrame:
    """
    Export the default raw-value table for the standard setup.
    """
    table = create_raw_value_table(
        df=df,
        x_config=x_config,
        x_values=[1, 16, 256, 4096, 65536],
        strategies=["clone", "slice", "rc"],
        metric_col=metric_col,
    )

    csv_path = folder_path / f"{metric_col}_raw_table.csv"
    table.to_csv(csv_path)

    print(f"Raw table exported to: {csv_path}")
    return table


def size_change(csv_path):
    analysis_path = Path(csv_path)
    df = pd.read_csv(analysis_path)
    df = add_throughputs(df)
    folder_path = analysis_path.parent

    df = add_relative_change_to_previous(
        df,
        metric_col="thr_mean",
        x_config=x_config,
        group_cols=["strategy"],
        relative_col="thr_mean_rel_prev",
    )

    plot_relative_throughputs_prev(df, folder_path)
    plot_mean_throughputs(df, folder_path)
    plot_mean_throughputs_log(df, folder_path)
    plot_mean_latencies(df, folder_path)
    plot_relative_throughputs_to_slice(df, folder_path)
    plot_relative_throughputs(df, folder_path)
    plot_memory_usage(df, folder_path)
    export_default_baseline_comparison(df, folder_path, "thr_mean")
    export_default_raw_table(df, folder_path, "mem_total")
