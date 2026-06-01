from __future__ import annotations

from pathlib import Path

import pandas as pd

import overview_plotters
import plot_configs
from generate_plots import add_throughputs, add_relative_metric

x_config = plot_configs.x_perc_overlap(descending=True)


def plot_relative_throughputs_vary_overlap(
        df: pd.DataFrame,
        folder_path,
):
    df = add_relative_metric(
        df,
        metric_col="thr_mean",
        x_config=x_config,
    )

    y_config = plot_configs.y_thr_mean_rel(y_log=True)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["rc", "slice", "clone", "legacy"],
    )
    plotter(df, folder_path)

    y_config = plot_configs.y_thr_mean_rel(y_log=False)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["rc", "slice", "clone", "legacy"],
    )
    plotter(df, folder_path)


def plot_absolute_throughputs_vary_overlap(
        df: pd.DataFrame,
        folder_path,
):
    y_config = plot_configs.y_thr_mean(y_log=True)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["rc", "slice", "clone", "legacy"],
    )
    plotter(df, folder_path)

    y_config = plot_configs.y_thr_mean(y_log=False)
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["rc", "slice", "clone", "legacy"],
    )
    plotter(df, folder_path)


def vary_overlap(csv_path: str):
    analysis_path = Path(csv_path)
    df = pd.read_csv(analysis_path)
    df = add_throughputs(df)
    folder_path = analysis_path.parent

    plot_relative_throughputs_vary_overlap(df, folder_path)
    plot_absolute_throughputs_vary_overlap(df, folder_path)


if __name__ == "__main__":
    vary_overlap("evaluation/vary_overlap/summary.csv")
