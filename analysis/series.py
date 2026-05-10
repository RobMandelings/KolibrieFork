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
    return statistics.mean((strat_data["throughput"]["sample_thr"]))


def get_thr_med_from_strat(strat_data):
    return statistics.median((strat_data["throughput"]["sample_thr"]))


def get_mean_ns_from_strat(strat_data):
    return statistics.mean((strat_data["throughput"]["sample_ns"]))


def get_med_ns_from_strat(strat_data):
    return statistics.median((strat_data["throughput"]["sample_ns"]))


def get_thr_std_dev_from_strat(strat_data):
    sample = strat_data["throughput"]["sample_thr"]
    return statistics.stdev(sample)


def perc_overlap_label(row: pd.Series) -> str:
    size = row.get("window.size")
    slide = row.get("window.slide")

    if pd.isna(size) or size == 0 or pd.isna(slide):
        return "N/A"

    perc_overlap = (size - slide) / size
    if perc_overlap < 0:
        perc_overlap = 0

    return f"{perc_overlap:.2%}"


def slide_label(row: pd.Series) -> str:
    slide = row.get("window.slide")
    return f"{slide}"


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
        reserve = workload.get("reserve")

        for strat_name, strat_data in strategies.items():
            # ---- Hardcoded retrieval of result metrics ----
            # Here you inline the logic from your existing value_getters.
            #
            # Example placeholders – replace with your actual implementations.
            thr_mean = get_thr_mean_from_strat(strat_data)
            thr_median = get_thr_med_from_strat(strat_data)
            ns_mean = get_mean_ns_from_strat(strat_data)
            ns_median = get_med_ns_from_strat(strat_data)
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
                "sec_mean": ns_mean,
                "sec_median": ns_median,
                "thr_std_dev": thr_std_dev,
                "mem_total": mem_total,
                "reserve": reserve
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

        reserve = workload.get("reserve")

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
                    "reserve": reserve
                })

    if not rows:
        raise ValueError("No samples found in workloads")

    return pd.DataFrame(rows)


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
