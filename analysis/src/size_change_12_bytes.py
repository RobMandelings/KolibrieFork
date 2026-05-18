from pathlib import Path

import pandas as pd

import overview_plotters
import plot_configs
from generate_plots import add_throughputs, add_relative_to_slice


def plot_relative_throughputs(df: pd.DataFrame, x_config: plot_configs.XConfig, folder_path):
    y_config = plot_configs.y_thr_mean(y_log=False)
    df = add_relative_to_slice(df,
                               strategy_name="slice",
                               metric_col="thr_mean")
    plotter = overview_plotters.make_strategy_comparison_plotter(
        x_config,
        y_config,
        strategies=["clone", "slice", "rc", "arc"]
    )
    plotter(df, folder_path)


def size_change_12_bytes():
    analysis_path = Path("evaluation/size_change_12_bytes/summary.csv")
    df = pd.read_csv(analysis_path)
    df = add_throughputs(df)
    folder_path = analysis_path.parent

    x_config = plot_configs.x_size(False)

    plot_relative_throughputs(df, x_config, folder_path)


if __name__ == "__main__":
    size_change_12_bytes()
