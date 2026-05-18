from typing import Iterable

import pandas as pd
from scipy.stats import linregress
import plot_configs


def report_strategy_regressions(
        df: pd.DataFrame,
        x_config: plot_configs.XConfig,
        y_config: plot_configs.YConfig,  # whatever type you use for Y configs
        strategies: Iterable[str] = ("clone", "slice", "rc", "arc"),
) -> None:
    """
    For each strategy, take the series defined by x_config and y_config,
    run a simple linear regression, and print slope, intercept, r-value,
    p-value, and standard error.

    The regression is done on the same x/y columns that would be used
    for plotting.
    """
    x_col = x_config.workload_index_col
    y_col = y_config.y_col  # adjust if your Y config exposes the column differently

    # Ensure consistent ordering before regression
    df_sorted = df.sort_values(
        by=["strategy", x_col],
        ascending=[True, not x_config.descending],
    )

    for strat in strategies:
        sub = df_sorted[df_sorted["strategy"] == strat]

        # Drop rows where x or y is missing
        sub = sub[[x_col, y_col]].dropna()
        if sub.empty:
            print(f"[regression] strategy={strat}: no data")
            continue

        x = sub[x_col].to_numpy(dtype=float)
        y = sub[y_col].to_numpy(dtype=float)

        if len(x) < 2:
            print(f"[regression] strategy={strat}: not enough points ({len(x)})")
            continue

        res = linregress(x, y)

        print(
            f"[regression] strategy={strat} "
            f"x={x_col} y={y_col} "
            f"slope={res.slope:.6g} intercept={res.intercept:.6g} "
            f"r={res.rvalue:.4f} p={res.pvalue:.3g} stderr={res.stderr:.6g}"
        )