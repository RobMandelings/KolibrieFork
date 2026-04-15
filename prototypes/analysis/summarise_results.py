import math

from typing import Dict, List, Tuple

import throughput_results
import mem_results as mem_res_module
import pandas as pd
import re
import numpy as np
import matplotlib.pyplot as plt


def parse_size(key: str) -> int:
    """Extract integer window size from 'size=1,slide=1,offset=0'."""
    m = re.search(r"size\s*=\s*(\d+)", key)
    if not m:
        raise ValueError(f"Cannot parse size from key: {key}")
    return int(m.group(1))


def metric_table(dfs_by_window: Dict[str, pd.DataFrame], column: str) -> pd.DataFrame:
    """
    Return a table with:
      rows   = strategies (df index)
      cols   = window sizes (parsed from key)
      values = df.loc[strategy, column]
    """
    # sort by numeric size
    items = sorted(dfs_by_window.items(), key=lambda kv: parse_size(kv[0]))
    sizes = [parse_size(k) for k, _ in items]

    # infer strategies from any one dataframe
    some_df = next(iter(dfs_by_window.values()))
    strategies = list(some_df.index)

    data = {size: [] for size in sizes}
    for size, (_, df) in zip(sizes, items):
        for strat in strategies:
            try:
                val = df.loc[strat, column]
            except KeyError:
                val = np.nan
            data[size].append(val)

    return pd.DataFrame(data, index=strategies)


def plot_property(dfs_by_window: Dict[str, pd.DataFrame], column: str):
    table = metric_table(dfs_by_window, column)  # strategies x sizes

    sizes = table.columns.to_list()
    for strat, row in table.iterrows():
        plt.plot(sizes, row.values, marker="o", label=strat)

    plt.xlabel("size")
    plt.ylabel(column)
    plt.title(f"{column} vs window size")
    plt.legend()
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    plt.show()


def get_dfs_by_config(combined) -> Dict[str, "pd.DataFrame"]:
    rows = []
    for config_key, strategies in combined.items():
        for strat_name, strat_data in strategies.items():
            mem_dict = strat_data["memory"]
            thr_metrics = strat_data["throughput"]

            if "window_closed" in mem_dict:
                mem_metrics = mem_dict["window_closed"]
                mem_total_bytes = mem_metrics.total_bytes
                mem_total_blocks = mem_metrics.total_blocks
                mem_max_bytes = mem_metrics.t_gmax_bytes
                mem_max_blocks = mem_metrics.t_gmax_blocks
            else:
                mem_total_bytes = math.nan
                mem_total_blocks = math.nan
                mem_max_bytes = math.nan
                mem_max_blocks = math.nan

            row = {
                "config": config_key,
                "strategy": strat_name,
                "mem_total_bytes": mem_total_bytes,
                "mem_total_blocks": mem_total_blocks,
                "mem_max_bytes": mem_max_bytes,
                "mem_max_blocks": mem_max_blocks,
                "nr_elements": thr_metrics["nr_elements"],
                "time_mean_ns": thr_metrics["mean"]["point_estimate"],
                "time_median_ns": thr_metrics["median"]["point_estimate"],
                "time_std_dev_ns": thr_metrics["std_dev"]["point_estimate"]
            }
            rows.append(row)

    df = pd.DataFrame(rows)
    df.set_index(["config", "strategy"], inplace=True)

    configs = df.index.get_level_values("config").unique()

    dfs_by_config = {
        cfg: df.xs(cfg, level="config").copy()
        for cfg in configs
    }

    for cfg, df in dfs_by_config.items():
        df["thr_mean_elem_per_s"] = df["nr_elements"] / (df["time_mean_ns"] * 1e-9)

    baseline = df["thr_mean_elem_per_s"].max()  # global max baseline
    for cfg, df in dfs_by_config.items():
        df["thr_mean_elem_rel"] = df["thr_mean_elem_per_s"] / baseline

    return dfs_by_config


def filter_by_nr_elements(results, n):
    return {
        name: data
        for name, data in results.items()
        if "10000" in name
    }


""" Mapping between the criterion benchmark tests and dheap things """
NAME_MAPPING = {
    "clone": "CloneStrategy",
    "expire": "ExpireStrategy",
    "refcount": "RefCountStrategy"
}


def parse_workload_key(key: str) -> dict:
    parts = key.split("_")
    out = {}
    for part in parts:
        if "=" not in part:
            continue  # ignore pieces that can't be split
        k, v = part.split("=", 1)
        try:
            out[k] = int(v)
        except ValueError:
            out[k] = v
    return out


def filter_by_events(results: dict, target_events: int) -> dict:
    return {
        key: value
        for key, value in results.items()
        if parse_workload_key(key).get("events") == target_events
    }


def select_labeled_dfs(
        dfs_by_config: Dict[str, pd.DataFrame],
        workload_names: List[str],
) -> List[pd.DataFrame]:
    """
    Returns a list of (label, DataFrame) for plotting.
    name_to_label maps workload_name -> label used in the plot.
    """
    pairs: List[pd.DataFrame] = []

    for workload_name in workload_names:
        df = dfs_by_config.get(workload_name)
        if df is None:
            continue  # or raise if you want strictness
        pairs.append(df)

    return pairs


def build_overview_from_dfs(
        dfs: List[pd.DataFrame],
        labels: List[str],
        prop: str,
) -> pd.DataFrame:
    """
    dfs: list of DataFrames indexed by strategy.
    labels: same-length list of column labels for each df.
    Returns a DataFrame indexed by strategy with one column per label,
    containing df[prop] for that label.
    """
    if len(dfs) != len(labels):
        raise ValueError("dfs and labels must have the same length")

    base_df = None

    for label, df in zip(labels, dfs):
        col = df[[prop]].rename(columns={prop: label})
        if base_df is None:
            base_df = col
        else:
            base_df = base_df.join(col, how="outer")

    if base_df is None:
        return pd.DataFrame()

    return base_df


def plot_overview(overview, ylabel, log_scale=False, strategies=None, title=None):
    if strategies is None:
        strategies = overview.index.tolist()

    x = list(overview.columns)

    # Choose different colors for strategies
    colors = {
        "clone": "tab:blue",
        "refcount": "tab:red",
        "arc": "tab:orange",
        "expire": "tab:green",
        "legacy": "tab:purple"
    }

    plt.figure(figsize=(16, 6))

    for strategy in overview.index:

        if strategy in strategies:
            y = overview.loc[strategy].values
            plt.plot(
                x,
                y,
                marker="o",
                label=strategy,
                color=colors.get(strategy, None),
            )

    plt.xlabel("window size label")  # or something more specific
    plt.xticks(rotation=45, ha="right", fontsize=6)
    plt.ylabel(ylabel)
    if log_scale:
        plt.yscale("log")
    if title is not None:
        plt.title(title)

    plt.legend()
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    plt.show()


def load_results(path: str) -> dict:
    strats = mem_res_module.load_mem_results(path)
    thr_results = throughput_results.load_estimates(path)
    combined_mem_throughput = {}

    for workload_name, strat_results in strats.items():
        combined_mem_throughput[workload_name] = {}

        if workload_name not in strats or workload_name not in thr_results:
            print(f"Ignoring workload {workload_name}: not present in both throughput and mem results")
            continue

        for strat_name, mem_result in strat_results.items():
            # pick the matching throughput result
            thr_result = thr_results[workload_name][strat_name]

            combined_mem_throughput[workload_name][strat_name] = {
                "memory": mem_result,
                "throughput": thr_result,
            }

    return combined_mem_throughput


def main():
    results = load_results("15_04")
    dfs_by_config = get_dfs_by_config(results)

    name_label_pairs = []
    for size in [1, 2, 4, 8, 16]:
        for slide in [1]:
            for events in [50_000]:
                name_label_pairs.append(
                    (f"windows=1,size={size},slide={slide},events={events}", f"1,{size},{slide},{events}"))

    names = [name for name, _ in name_label_pairs]
    labels = [label for _, label in name_label_pairs]

    if len(dfs_by_config.keys()) != len(labels):
        print("name_label pairs is specified incorrectly. Number of labels is not the same as the number of configurations.")
        return

    selected_dfs = select_labeled_dfs(dfs_by_config, names)

    overview_df = build_overview_from_dfs(selected_dfs, labels, "thr_mean_elem_rel")
    plot_overview(overview_df, "thr_mean_elem_rel", False)

    overview_df = build_overview_from_dfs(selected_dfs, labels, "mem_total_blocks")
    plot_overview(overview_df, "mem_total_blocks", False)

    overview_df = build_overview_from_dfs(selected_dfs, labels, "mem_total_bytes")
    plot_overview(overview_df, "mem_total_bytes", False)


if __name__ == "__main__":
    main()
