from pathlib import Path

import pandas as pd

from arg_parser import parse_args
from overview_plotters import make_default_overview_plotters, PlotVariant, Y_THR_MEAN, Y_THR_MEAN_REL, Y_MEM


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


def decorate_df(df: pd.DataFrame, x_variant: PlotVariant, descending=False):
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


def generate_plots(analysis_path: Path):
    x_variant = PlotVariant.PERC_OVERLAP
    descending = False
    df = pd.read_csv(analysis_path / "csv" / "summary.csv")
    df = decorate_df(df, x_variant, descending)
    plotters = make_default_overview_plotters(PlotVariant.PERC_OVERLAP, Y_MEM, descending)
    for plotter in plotters:
        plotter(df, analysis_path)


def main() -> None:
    args = parse_args()
    analysis_path = Path(args.target).resolve()
    print(f"Using analysis path: {analysis_path}")
    generate_plots(analysis_path)


if __name__ == "__main__":
    main()
