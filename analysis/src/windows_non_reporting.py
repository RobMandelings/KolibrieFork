import pandas as pd

import plot_configs
from regression import report_strategy_regressions


def windows_non_reporting():
    analysis_path = "../evaluation/windows_non_reporting/summary.csv"
    df = pd.read_csv(analysis_path)
    x_config = plot_configs.x_nr_windows(False)
    y_config = plot_configs.Y_LAT_MEAN
    report_strategy_regressions(df, x_config, y_config)


if __name__ == "__main__":
    windows_non_reporting()
