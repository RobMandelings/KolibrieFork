import math
from typing import Dict

import mem_results as mem_res_module
import throughput_results
import pandas as pd


def _load_raw_results(path: str) -> dict:
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


def _get_dfs_by_workload(raw_results: dict) -> Dict[str, "pd.DataFrame"]:
    """
    Returns dictionary where the key is a specific workload and the value is a pandas DataFrame.
    """
    rows = []
    for config_key, strategies in raw_results.items():
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

    dfs_per_workload = {
        cfg: df.xs(cfg, level="config").copy()
        for cfg in configs
    }

    for cfg, df in dfs_per_workload.items():
        df["thr_mean_elem_per_s"] = df["nr_elements"] / (df["time_mean_ns"] * 1e-9)

    baseline = df["thr_mean_elem_per_s"].max()  # global max baseline
    for cfg, df in dfs_per_workload.items():
        df["thr_mean_elem_rel"] = df["thr_mean_elem_per_s"] / baseline

    return dfs_per_workload


def get_results(path: str) -> Dict[str, pd.DataFrame]:
    raw_results = _load_raw_results(path)
    return _get_dfs_by_workload(raw_results)