import statistics
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Optional, Any, Dict

import matplotlib.pyplot as plt
import pandas as pd
from scipy.stats import stats

from constants import STRATEGY_COLORS, STRATEGY_MARKERS
from linreg import linear_trend_per_strategy


@dataclass
class PlotterConfig:
    title: str
    value_getter: Callable[[dict, str, dict], Any]
    x_label_getter: Callable[[dict, str, dict], str]
    x_label: str
    y_label: str
    subdir: Path
    error_getter: Optional[Callable[[dict, str, dict], Any]] = None


def get_total_bytes_for_strat(strat_data):
    return strat_data["memory"]["total_bytes"]["TOTAL"]


def get_thr_mean_from_strat(strat_data):
    return statistics.mean((strat_data["throughput"]["sample"]))


def get_thr_med_from_strat(strat_data):
    return statistics.median((strat_data["throughput"]["sample"]))


def get_thr_std_dev_from_strat(strat_data):
    sample = strat_data["throughput"]["sample"]
    return statistics.stdev(sample)


def add_perc_overlap_label(df: pd.DataFrame, out_col: str = "x_label") -> pd.DataFrame:
    df = df.copy()

    def compute_label(row):
        size = row.get("window.size")
        slide = row.get("window.slide")

        if pd.isna(size) or size == 0 or pd.isna(slide):
            return "N/A"

        perc_overlap = (size - slide) / size
        if perc_overlap < 0:
            perc_overlap = 0

        return f"{perc_overlap:.2%}"

    df[out_col] = df.apply(compute_label, axis=1)
    return df


def workloads_to_dataframe(workloads: dict) -> pd.DataFrame:
    """
    Transform workloads dict into a flat DataFrame with one row per (workload, strategy).

    Expected structure of each workload entry:
    workloads = {
        workload_key: {
            "workload": {
                "nr_events": int,
                "stream_config": {
                    "spread": int,
                    "offset": int,
                },
                "bytes": int,
                "window": {
                    "size": int,
                    "slide": int,
                    "offset": int,
                },
                "nr_windows": int,
                # "events_per_report": float,  # intentionally ignored
            },
            "strategies": {
                strategy_name: {
                    # whatever structure your result getters use
                    # for throughput and memory statistics
                }
            }
        },
        ...
    }

    Returns a DataFrame with columns:
    [
        "strategy",
        "nr_events",
        "stream_config.spread",
        "stream_config.offset",
        "bytes",
        "window.size",
        "window.slide",
        "window.offset",
        "nr_windows",
        "thr_mean",
        "thr_median",
        "thr_std_dev",
        "mem_total",
    ]
    """
    rows = []

    for workload_key, workload_entry in workloads.items():
        workload = workload_entry["workload"]
        strategies = workload_entry.get("strategies", {})

        # Extract workload-level parameters
        nr_events = workload.get("nr_events")
        stream_cfg = workload.get("stream_config", {})
        stream_spread = stream_cfg.get("spread")
        stream_offset = stream_cfg.get("offset")

        bytes_ = workload.get("bytes")

        window_cfg = workload.get("window", {})
        win_size = window_cfg.get("size")
        win_slide = window_cfg.get("slide")
        win_offset = window_cfg.get("offset")

        nr_windows = workload.get("nr_windows")

        for strat_name, strat_data in strategies.items():
            # ---- Hardcoded retrieval of result metrics ----
            # Here you inline the logic from your existing value_getters.
            #
            # Example placeholders – replace with your actual implementations.
            thr_mean = get_thr_mean_from_strat(strat_data)
            thr_median = get_thr_med_from_strat(strat_data)
            thr_std_dev = get_thr_std_dev_from_strat(strat_data)
            mem_total = get_total_bytes_for_strat(strat_data)

            row = {
                "strategy": strat_name,
                "nr_events": nr_events,
                "stream_config.spread": stream_spread,
                "stream_config.offset": stream_offset,
                "bytes": bytes_,
                "window.size": win_size,
                "window.slide": win_slide,
                "window.offset": win_offset,
                "nr_windows": nr_windows,
                "thr_mean": thr_mean,
                "thr_median": thr_median,
                "thr_std_dev": thr_std_dev,
                "mem_total": mem_total,
            }

            rows.append(row)

    if not rows:
        raise ValueError("No data found in workloads to build DataFrame")

    df = pd.DataFrame(rows)
    return df


def workloads_samples_to_dataframe(workloads: dict) -> pd.DataFrame:
    """
    Build a long DataFrame with one row per throughput sample across all
    (workload, strategy) combinations.

    Columns:
      - workload_index
      - strategy
      - nr_events
      - stream_config.spread
      - stream_config.offset
      - bytes
      - window.size
      - window.slide
      - window.offset
      - nr_windows
      - thr_sample
    """
    rows = []

    for workload_index, (workload_key, entry) in enumerate(workloads.items()):
        workload = entry["workload"]
        strategies = entry.get("strategies", {})

        # Workload-level parameters (same as your summary)
        nr_events = workload.get("nr_events")
        stream_cfg = workload.get("stream_config", {})
        stream_spread = stream_cfg.get("spread")
        stream_offset = stream_cfg.get("offset")

        bytes_ = workload.get("bytes")

        window_cfg = workload.get("window", {})
        win_size = window_cfg.get("size")
        win_slide = window_cfg.get("slide")
        win_offset = window_cfg.get("offset")

        nr_windows = workload.get("nr_windows")

        for strategy_name, strat_data in strategies.items():
            # Get the throughput samples for this (workload, strategy)
            samples = strat_data.get("throughput", {}).get("sample", [])

            for sample in samples:
                rows.append({
                    "workload_index": workload_index,
                    "strategy": strategy_name,
                    "nr_events": nr_events,
                    "stream_config.spread": stream_spread,
                    "stream_config.offset": stream_offset,
                    "bytes": bytes_,
                    "window.size": win_size,
                    "window.slide": win_slide,
                    "window.offset": win_offset,
                    "nr_windows": nr_windows,
                    "thr_sample": sample,
                })

    if not rows:
        raise ValueError("No samples found in workloads")

    return pd.DataFrame(rows)


def build_strategy_series(config: PlotterConfig, workloads):
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
            value = config.value_getter(workloads, workload_key, strat_name)
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

    if not series:
        raise Exception

    return series


def add_regression_overlay(ax, strategy, x_values, regression_results):
    reg = regression_results.get(strategy)
    if reg is None or len(x_values) < 2:
        return

    y_fit = [reg["intercept"] + reg["slope"] * x for x in x_values]

    ax.plot(
        x_values,
        y_fit,
        linestyle="--",
        linewidth=0.5,
        color=STRATEGY_COLORS[strategy],
        alpha=0.9,
    )


def plot_strategy_series(
        workloads,
        series,
        xlabel: str,
        ylabel: str,
        strategies=None,
        title=None,
        output_file=None,
        workload_order=None,
        overlay_events_per_report=False,
        remove_outliers=False,
):
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

    fig, ax1 = plt.subplots(figsize=(16, 6))

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

        regression_results = linear_trend_per_strategy(series, workload_order)
        add_regression_overlay(
            ax=ax1,
            strategy=strategy,
            x_values=cur_x_pos,
            regression_results=regression_results,
        )

        if has_any_error:
            if all(
                    err is not None and not isinstance(err, (list, tuple))
                    for err in y_errors
            ):
                ax1.errorbar(
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
                ax1.errorbar(
                    cur_x_pos,
                    y_values,
                    yerr=[lower, upper],
                    fmt=f"-{marker}",
                    label=strategy,
                    color=color,
                    capsize=4,
                )
            else:
                ax1.plot(
                    cur_x_pos,
                    y_values,
                    marker=marker,
                    label=strategy,
                    color=color,
                )
        else:
            ax1.plot(
                cur_x_pos,
                y_values,
                marker=marker,
                label=strategy,
                color=color,
            )

    ax1.set_xlabel(xlabel)
    ax1.set_xticks(x_pos)
    ax1.set_xticklabels(x_tick_labels, rotation=45, ha="right", fontsize=8)
    ax1.set_ylabel(ylabel, fontsize=14)

    ax2 = None
    if overlay_events_per_report and workloads is not None:
        overlay_x = []
        overlay_y = []

        for i, workload_key in enumerate(workload_order):
            workload_data = workloads.get(workload_key, {})
            workload = workload_data.get("workload", {})
            epr = workload.get("events_per_report")

            if epr is None:
                continue

            overlay_x.append(i)
            overlay_y.append(epr)

        if overlay_x:
            ax2 = ax1.twinx()
            ax2.plot(
                overlay_x,
                overlay_y,
                color="black",
                marker="o",
                linestyle="--",
                linewidth=2,
                label="Events per report",
            )
            ax2.set_ylabel("Events per report", fontsize=14)

    if title is not None:
        ax1.set_title(title, fontsize=16)

    ax1.grid(True, alpha=0.3)

    handles1, labels1 = ax1.get_legend_handles_labels()
    if ax2 is not None:
        handles2, labels2 = ax2.get_legend_handles_labels()
        ax1.legend(handles1 + handles2, labels1 + labels2)
    else:
        ax1.legend()

    fig.tight_layout()

    if output_file is not None:
        output_file = Path(output_file)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        fig.savefig(output_file, dpi=200, bbox_inches="tight")
        plt.close(fig)
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


def linear_trend_per_strategy_df(
        df: pd.DataFrame,
        y_col: str,
        descending: bool,
        strategy_col: str = "strategy",
        x_order_col: str = "window.slide",
) -> Dict[str, Dict[str, float]]:
    """
    For each strategy in df, run a simple linear regression of
    y = y_col vs. x = position in the sorted unique values of x_order_col.

    Parameters
    ----------
    df : pd.DataFrame
        Must contain at least strategy_col, x_order_col, and y_col.
        Typically one row per (workload, strategy).
    y_col : str
        Column to regress on, e.g. 'thr_mean', 'thr_median', 'mem_total'.
    strategy_col : str
        Column that identifies the strategy.
    x_order_col : str
        Column used to define x-axis ordering, e.g. 'window.slide'.

    Returns
    -------
    dict
        strategy -> { "slope": float, "intercept": float,
                      "p_value": float, "r_value": float }
    """
    # Global x-order: sorted unique x values (e.g. slide values)
    workload_order = (
        df[x_order_col]
            .drop_duplicates()
            .sort_values(ascending=not descending)
            .tolist()
    )
    order_to_x = {value: i for i, value in enumerate(workload_order)}

    results: Dict[str, Dict[str, float]] = {}

    for strategy, strat_df in df.groupby(strategy_col):
        x_vals = []
        y_vals = []

        # For each possible x value in order, see if this strategy has a row
        for order_value in workload_order:
            row_df = strat_df[strat_df[x_order_col] == order_value]
            if row_df.empty:
                continue

            row = row_df.iloc[0]

            if pd.isna(row[y_col]):
                continue

            x_vals.append(float(order_to_x[order_value]))  # 0, 1, 2, ...
            y_vals.append(float(row[y_col]))

        if len(x_vals) < 2:
            continue

        slope, intercept, r_value, p_value, std_err = stats.linregress(x_vals, y_vals)

        results[strategy] = {
            "slope": slope,
            "intercept": intercept,
            "p_value": p_value,
            "r_value": r_value,
        }

    return results
