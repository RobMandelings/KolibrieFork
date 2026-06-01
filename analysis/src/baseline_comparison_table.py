import pandas as pd
from typing import List


def format_large_value(value) -> str:
    if pd.isna(value):
        return "N/A"

    value = float(value)
    abs_value = abs(value)

    if abs_value >= 1_000_000_000:
        return f"{value / 1_000_000_000:.2f}B"
    if abs_value >= 1_000_000:
        return f"{value / 1_000_000:.2f}M"
    if abs_value >= 1_000:
        return f"{value / 1_000:.2f}K"

    if value.is_integer():
        return str(int(value))

    return f"{value:.2f}"


def create_raw_value_table(
        df: pd.DataFrame,
        x_config,
        x_values: List,
        strategies: List[str],
        metric_col: str,
) -> pd.DataFrame:
    """
    Create a table showing raw values for a metric at selected x-values.
    """
    x_col = x_config.workload_index_col

    filtered_df = df[
        (df[x_col].isin(x_values)) &
        (df["strategy"].isin(strategies))
    ].copy()

    pivot = filtered_df.pivot_table(
        values=metric_col,
        index="strategy",
        columns=x_col,
        aggfunc="mean"
    )

    ordered_strategies = ["slice"] + [
        s for s in strategies if s != "slice"
    ]

    table = pd.DataFrame(index=ordered_strategies, columns=x_values)

    for strategy in ordered_strategies:
        if strategy not in pivot.index:
            continue

        for x_val in x_values:
            if x_val not in pivot.columns:
                table.loc[strategy, x_val] = "N/A"
                continue

            value = pivot.loc[strategy, x_val]
            table.loc[strategy, x_val] = format_large_value(value)

    return table


def create_baseline_comparison_table(
        df: pd.DataFrame,
        x_config,
        x_values: List,
        strategies: List[str],
        baseline_strategy: str,
        metric_col: str = "thr_mean",
) -> pd.DataFrame:
    """
    Create a table showing relative performance compared to a baseline strategy.

    Baseline values are shown as 100\\%, and other strategies are shown as
    percentages relative to that baseline.

    Parameters
    ----------
    df : pd.DataFrame
        The input dataframe containing benchmark results.
    x_config : XConfig
        The x-axis configuration (e.g. plot_configs.x_nr_elems()).
    x_values : List
        List of x-axis values to include (e.g. [1, 16, 256]).
    strategies : List[str]
        List of strategy names to compare.
    baseline_strategy : str
        The baseline strategy to compare against.
    metric_col : str
        The metric column to compare (default: "thr_mean").

    Returns
    -------
    pd.DataFrame
        Table with strategies as rows, x_values as columns, and relative
        percentages as LaTeX-safe percentage strings.
    """
    x_col = x_config.workload_index_col

    filtered_df = df[
        (df[x_col].isin(x_values)) &
        (df["strategy"].isin(strategies))
    ].copy()

    pivot = filtered_df.pivot_table(
        values=metric_col,
        index="strategy",
        columns=x_col,
        aggfunc="mean"
    )

    if baseline_strategy not in pivot.index:
        raise ValueError(f"Baseline strategy '{baseline_strategy}' not found in data")

    baseline_values = pivot.loc[baseline_strategy]

    ordered_strategies = [baseline_strategy] + [
        s for s in strategies if s != baseline_strategy
    ]

    percentage_table = pd.DataFrame(index=ordered_strategies, columns=x_values)

    for strategy in ordered_strategies:
        if strategy not in pivot.index:
            continue
        for x_val in x_values:
            if x_val not in pivot.columns:
                percentage_table.loc[strategy, x_val] = "N/A"
                continue

            strategy_value = pivot.loc[strategy, x_val]
            baseline_value = baseline_values[x_val]

            if (
                pd.notna(strategy_value)
                and pd.notna(baseline_value)
                and baseline_value != 0
            ):
                relative_pct = (strategy_value / baseline_value) * 100
                percentage_table.loc[strategy, x_val] = f"{relative_pct:.2f}\\%"
            else:
                percentage_table.loc[strategy, x_val] = "N/A"

    percentage_table.index.name = "variant"

    return percentage_table
