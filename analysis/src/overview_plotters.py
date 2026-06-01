from __future__ import annotations

import pandas as pd

from constants import STRATEGIES
from overview_plotting import make_overview_plotter, make_strategy_windows_overview_plotter, PlotFontConfig
from plot_configs import YConfig, XConfig


def identity_preprocess(df: pd.DataFrame) -> pd.DataFrame:
    return df


def make_strategy_comparison_plotter(
        x_config: XConfig,
        y_config: YConfig,
        strategies: list[str] | None,
        title: str | None = None,
        suffix: str | None = None,
        font_config: PlotFontConfig | None = None,
):
    """
    Creates one plotter that compares the given strategies with each other.

    - strategies=None means compare all strategies
    - strategies=["slice", "clone"] means compare only those two
    """
    base_title = title or y_config.default_title

    if strategies is None:
        resolved_suffix = suffix or "all"
        strategy_title = "all"
    else:
        resolved_suffix = suffix or "_".join(strategies)
        strategy_title = ", ".join(strategies)

    return make_overview_plotter(
        x_config=x_config,
        y_config=y_config,
        title=f"{base_title} ({strategy_title})",
        output_file=y_config.subdir / f"{y_config.filename}_{resolved_suffix}.png",
        strategies=strategies,
        font_config=font_config
    )


def make_all_strategy_comparison_plotter(
        x_config: XConfig,
        y_config: YConfig,
        title: str | None = None,
):
    return make_strategy_comparison_plotter(
        x_config=x_config,
        y_config=y_config,
        strategies=None,
        title=title,
        suffix="all",
    )


def make_selected_strategy_comparison_plotter(
        x_variant: XConfig,
        y_config: YConfig,
        strategies: list[str],
        title: str | None = None,
):
    return make_strategy_comparison_plotter(
        x_config=x_variant,
        y_config=y_config,
        strategies=strategies,
        title=title,
        suffix="_".join(strategies),
    )


# def make_default_overview_plotters(
#         x_variant: XConfig,
#         y_config: YConfig,
#         title: str | None = None,
# ):
#     """
#     Creates plots for each of the strategies individually + overall
#     """
#     plotters = []
#
#     base_title = title or y_config.default_title
#
#     for strategy in [None] + STRATEGIES:
#         if strategy is None:
#             suffix = "all"
#             strategies = None
#             strategy_title = "all"
#         else:
#             suffix = strategy
#             strategies = [strategy]
#             strategy_title = strategy
#
#         plotters.append(
#             make_overview_plotter(
#                 x_variant=x_variant,
#                 y_config=y_config,
#                 title=f"{base_title} ({strategy_title})",
#                 output_file=y_config.subdir / f"{y_config.filename}_{suffix}.png",
#                 strategies=strategies,
#             )
#         )
#
#     return plotters

def make_strategy_windows_overview_plotters(
        *,
        x_config: XConfig,
        y_config: YConfig,
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
                x_config=x_config,
                y_config=y_config,
                strategy=strategy,
                windows=windows,
                title=f"{base_title} ({strategy} windows)",
            )
        )

    return plotters
