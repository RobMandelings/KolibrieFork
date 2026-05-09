import pandas as pd

from constants import STRATEGIES
from overview_plotting import make_overview_plotter, make_strategy_windows_overview_plotter
from plot_configs import PlotVariant, YConfig


def identity_preprocess(df: pd.DataFrame) -> pd.DataFrame:
    return df


def make_default_overview_plotters(
        x_variant: PlotVariant,
        y_config: YConfig,
        descending: bool = False,
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
                y_col=y_config.y_col,
                yerr_col=y_config.yerr_col,
                title=f"{base_title} ({strategy_title})",
                xlabel=x_variant.x_label,
                ylabel=y_config.ylabel,
                workload_index_col=x_variant.workload_index_col,
                descending=descending,
                output_file=y_config.subdir / f"{y_config.filename}_{suffix}.png",
                label_fn=x_variant.label_fn,
                strategies=strategies,
            )
        )

    return plotters


def make_strategy_windows_overview_plotters(
        *,
        x_variant: PlotVariant,
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
                x_variant=x_variant,
                y_config=y_config,
                strategy=strategy,
                windows=windows,
                descending=descending,
                title=f"{base_title} ({strategy} windows)",
            )
        )

    return plotters
