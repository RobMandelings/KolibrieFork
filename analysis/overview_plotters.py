from __future__ import annotations

import pandas as pd

from constants import STRATEGIES
from overview_plotting import make_overview_plotter, make_strategy_windows_overview_plotter
from plot_configs import YConfig, XConfig


def identity_preprocess(df: pd.DataFrame) -> pd.DataFrame:
    return df

def make_default_overview_plotters(
        x_variant: XConfig,
        y_config: YConfig,
        title: str | None = None,
):
    plotters = []

    base_title = title or y_config.default_title

    for strategy in [None] + STRATEGIES:
        if strategy is None:
            suffix = "all"
            strategies = None
            strategy_title = "all"
        else:
            suffix = strategy
            strategies = [strategy]
            strategy_title = strategy

        plotters.append(
            make_overview_plotter(
                x_variant=x_variant,
                y_config=y_config,
                title=f"{base_title} ({strategy_title})",
                output_file=y_config.subdir / f"{y_config.filename}_{suffix}.png",
                strategies=strategies,
            )
        )

    return plotters


def make_strategy_windows_overview_plotters(
        *,
        x_variant: XConfig,
        y_config: YConfig,
        descending: bool = False,
        title: str | None = None,
        strategies: list[str] | None = None,
        windows: list[int] | None = None,
):
    plotters = []

    strategies_to_plot = strategies or STRATEGIES
    base_title = title or y_config.default_title

    for strategy in strategies_to_plot:
        plotters.append(
            make_strategy_windows_overview_plotter(
                x_config=x_variant,
                y_config=y_config,
                strategy=strategy,
                windows=windows,
                descending=descending,
                title=f"{base_title} ({strategy} windows)",
            )
        )

    return plotters
