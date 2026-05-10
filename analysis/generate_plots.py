from pathlib import Path

import pandas as pd

from arg_parser import parse_args
from overview_plotters import make_default_overview_plotters, make_strategy_windows_overview_plotters
import plot_configs


def add_relative_metric(
        df: pd.DataFrame,
        *,
        strategy_col: str,
        sort_key_col: str,
        metric_col: str,
        relative_col: str,
        ascending: bool = True,
) -> pd.DataFrame:
    """
    For each strategy, pick the first row by sort_key_col (after sorting),
    use its metric_col as baseline, and add relative_col = metric_col / baseline
    within that strategy group.
    """
    df = df.copy()

    # Sort so that "first" is well-defined
    df = df.sort_values(by=[strategy_col, sort_key_col], ascending=[True, ascending])

    # Baseline per strategy (first row after sort)
    baselines = (
        df
            .groupby(strategy_col, as_index=False)
            .agg({metric_col: "first"})
            .rename(columns={metric_col: f"baseline_{metric_col}"})
    )

    # Join baseline back on strategy
    df = df.merge(baselines, on=strategy_col, how="left")

    # Compute relative metric
    df[relative_col] = df[metric_col] / df[f"baseline_{metric_col}"]

    return df


def add_relative_metric_vs_windows_baseline(
        df: pd.DataFrame,
        *,
        metric_col: str,
        relative_col: str,
        nr_windows_col: str = "nr_windows",
        baseline_windows_value: int = 1,
        group_cols: list[str] | None = None,
) -> pd.DataFrame:
    """
    Add a relative metric column by comparing each row against the row with
    nr_windows == baseline_windows_value for the same configuration.

    The baseline match is done on group_cols, which should contain all columns
    that define the same configuration apart from nr_windows.
    """
    df = df.copy()

    if group_cols is None:
        raise ValueError("group_cols must be provided")

    baseline_df = (
        df[df[nr_windows_col] == baseline_windows_value][group_cols + [metric_col]]
            .copy()
            .rename(columns={metric_col: f"baseline_{metric_col}"})
    )

    df = df.merge(
        baseline_df,
        on=group_cols,
        how="left",
    )

    df[relative_col] = df[metric_col] / df[f"baseline_{metric_col}"]

    return df


def decorate_df_with_window_relative_metrics(df: pd.DataFrame) -> pd.DataFrame:
    df = df.copy()

    group_cols = [
        "strategy",
        "nr_events",
        "stream_config.spread",
        "stream_config.offset",
        "bytes",
        "window.size",
        "window.slide",
        "window.offset",
    ]

    df = add_relative_metric_vs_windows_baseline(
        df,
        metric_col="thr_mean",
        relative_col="thr_mean_rel_window",
        nr_windows_col="nr_windows",
        baseline_windows_value=1,
        group_cols=group_cols,
    )

    df = add_relative_metric_vs_windows_baseline(
        df,
        metric_col="thr_median",
        relative_col="thr_median_rel_window",
        nr_windows_col="nr_windows",
        baseline_windows_value=1,
        group_cols=group_cols,
    )

    return df


def decorate_df(df: pd.DataFrame, x_variant: plot_configs.PlotVariant, descending=False):
    df = add_relative_metric(df,
                             strategy_col="strategy",
                             sort_key_col=x_variant.workload_index_col,
                             metric_col="thr_mean",
                             relative_col="thr_mean_rel",
                             ascending=not descending
                             )

    df = add_relative_metric(df,
                             strategy_col="strategy",
                             sort_key_col=x_variant.workload_index_col,
                             metric_col="thr_median",
                             relative_col="thr_median_rel",
                             ascending=not descending
                             )
    return df


def generate_plots(df: pd.DataFrame, folder_path: Path):
    x_variant = plot_configs.PlotVariant.PERC_OVERLAP
    descending = True

    df = decorate_df(df, x_variant, descending)

    plotters = make_default_overview_plotters(
        plot_configs.PlotVariant.PERC_OVERLAP,
        plot_configs.Y_THR_MEAN_REL,
        descending,
    )

    for plotter in plotters:
        plotter(df, folder_path)


def generate_strategy_window_plots(df: pd.DataFrame, folder_path: Path):
    x_variant = plot_configs.PlotVariant.PERC_OVERLAP
    descending = False

    df = decorate_df_with_window_relative_metrics(df)

    plotters = make_strategy_windows_overview_plotters(
        x_variant=x_variant,
        y_config=plot_configs.Y_MEM,
        descending=descending,
        strategies=["clone"],
        windows=[1, 2, 5, 10],
    )

    for plotter in plotters:
        plotter(df, folder_path)


def main() -> None:
    args = parse_args()
    analysis_path = Path(args.target).resolve()
    print(f"Using analysis path: {analysis_path}")

    # Read the CSV once
    df = pd.read_csv(analysis_path)
    folder_path = analysis_path.parent

    # Call both plot generation functions
    generate_plots(df, folder_path)
    generate_strategy_window_plots(df, folder_path)


if __name__ == "__main__":
    main()
