from pathlib import Path
import matplotlib.pyplot as plt

from constants import STRATEGY_COLORS, STRATEGY_MARKERS

from dataclasses import dataclass
from typing import Callable, Optional, Any


@dataclass
class SeriesBuildConfig:
    value_getter: Callable[[dict, str, dict], Any]
    x_label_getter: Callable[[dict, str, dict], str]
    error_getter: Optional[Callable[[dict, str, dict], Any]] = None


def build_strategy_series_from_workloads(
        workloads: dict,
        config: SeriesBuildConfig,
):
    """
    Returns:
    {
        strategy_name: {
            workload_key: {
                "value": ...,
                "x_label": ...,
                "yerr": ...   # optional, can be scalar or [lower, upper]
            }
        }
    }

    value_getter(strat_data, workload_key, workload_data) -> numeric | None
    error_getter(strat_data, workload_key, workload_data) -> numeric | [lower, upper] | None
    x_label_getter(strat_data, workload_key, workload_data) -> str | None
    """
    series = {}

    for workload_key, workload_data in workloads.items():
        strategies = workload_data.get("strategies", {})

        for strat_name, strat_data in strategies.items():
            value = config.value_getter(strat_data, workload_key, workload_data)
            if value is None:
                continue

            if strat_name not in series:
                series[strat_name] = {}

            point = {
                "value": value,
                "x_label": (
                    config.x_label_getter(strat_data, workload_key, workload_data)
                    if config.x_label_getter is not None
                    else workload_key
                ),
            }

            if config.error_getter is not None:
                yerr = config.error_getter(strat_data, workload_key, workload_data)
                if yerr is not None:
                    point["yerr"] = yerr

            series[strat_name][workload_key] = point

    return series


def plot_strategy_series(
        series: dict,
        xlabel: str,
        ylabel: str,
        strategies=None,
        title=None,
        output_file=None,
        workload_order=None,
):
    if not series:
        return

    if strategies is None:
        strategies = list(series.keys())

    if workload_order is None:
        workload_order = []
        seen = set()
        for strat_points in series.values():
            for workload_key in strat_points.keys():
                if workload_key not in seen:
                    seen.add(workload_key)
                    workload_order.append(workload_key)

    x_pos = list(range(len(workload_order)))

    x_tick_labels = []
    for workload_key in workload_order:
        label = workload_key
        for strat_points in series.values():
            point = strat_points.get(workload_key)
            if point is not None:
                label = point.get("x_label", workload_key)
                break
        x_tick_labels.append(label)

    plt.figure(figsize=(16, 6))

    for strategy in strategies:
        if strategy not in series:
            continue

        strat_points = series[strategy]

        cur_x_pos = []
        y_values = []
        y_errors = []
        has_any_error = False

        for i, workload_key in enumerate(workload_order):
            point = strat_points.get(workload_key)
            if point is None:
                continue

            cur_x_pos.append(i)
            y_values.append(point["value"])

            if "yerr" in point:
                y_errors.append(point["yerr"])
                has_any_error = True
            else:
                y_errors.append(None)

        if not cur_x_pos:
            continue

        marker = STRATEGY_MARKERS.get(strategy, "o")
        color = STRATEGY_COLORS.get(strategy, None)

        if has_any_error:

            if all(
                    err is not None and not isinstance(err, (list, tuple))
                    for err in y_errors
            ):
                plt.errorbar(
                    cur_x_pos,
                    y_values,
                    yerr=y_errors,
                    fmt=f"-{marker}",
                    label=strategy,
                    color=color,
                    capsize=4,
                )
            elif all(
                    err is not None and isinstance(err, (list, tuple)) and len(err) == 2
                    for err in y_errors
            ):
                lower = [err[0] for err in y_errors]
                upper = [err[1] for err in y_errors]
                plt.errorbar(
                    cur_x_pos,
                    y_values,
                    yerr=[lower, upper],
                    fmt=f"-{marker}",
                    label=strategy,
                    color=color,
                    capsize=4,
                )
            else:
                plt.plot(
                    cur_x_pos,
                    y_values,
                    marker=marker,
                    label=strategy,
                    color=color,
                )
        else:
            plt.plot(
                cur_x_pos,
                y_values,
                marker=marker,
                label=strategy,
                color=color,
            )

    plt.xlabel(xlabel)
    plt.xticks(x_pos, x_tick_labels, rotation=45, ha="right", fontsize=12)
    plt.ylabel(ylabel, fontsize=14)

    if title is not None:
        plt.title(title, fontsize=16)

    plt.legend()
    plt.grid(True, alpha=0.3)
    plt.tight_layout()

    if output_file is not None:
        output_file = Path(output_file)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        plt.savefig(output_file, dpi=200, bbox_inches="tight")
        plt.close()
    else:
        plt.show()


def get_throughput_estimate(strat_data, estimate_key):
    throughput = strat_data.get("throughput")
    if throughput is None:
        return None

    estimates = throughput.get("estimates")
    if estimates is None:
        return None

    return estimates.get(estimate_key)


def make_throughput_series_config(estimate_key: str, x_label_getter, error_getter):
    return SeriesBuildConfig(
        value_getter=lambda strat_data, workload_key, workload_data:
        get_throughput_estimate(strat_data, estimate_key),
        error_getter=error_getter,
        x_label_getter=x_label_getter,
    )


def plot_throughput_from_workloads(
        workloads: dict,
        config: SeriesBuildConfig,
        xlabel: str,
        ylabel: str = "throughput (events/s)",
        strategies=None,
        title=None,
        output_file=None,
        workload_order=None,
):
    series = build_strategy_series_from_workloads(workloads, config)

    plot_strategy_series(
        series=series,
        xlabel=xlabel,
        ylabel=ylabel,
        strategies=strategies,
        title=title,
        output_file=output_file,
        workload_order=workload_order,
    )
    print("Hello")
