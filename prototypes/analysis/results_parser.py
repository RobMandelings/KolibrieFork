import math
from pathlib import Path
from typing import Dict, Sequence, Any

import mem_results as mem_res_module
import throughput_results
import pandas as pd


def _load_raw_results(path: Path) -> dict:
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


def add_sample_throughputs(result: Dict[str, Dict[str, Dict[str, Any]]]) -> None:
    """
    For each workload and strategy in `result`, compute per-sample throughput
    from throughput["nr_elements"] and throughput["sample"]["times"/"iters"],
    and store it as throughput["sample"]["throughputs"] (elements per second).

    Assumes:
      - result[workload][strategy]["throughput"]["nr_elements"] exists
      - result[workload][strategy]["throughput"]["sample"]["times"] and
        ["iters"] exist and have the same length
      - 'times' are in nanoseconds for total sample
    """

    for workload_key, strategies in result.items():
        for strat_name, strat_data in strategies.items():
            thr = strat_data.get("throughput")
            if not thr:
                continue

            nr_elements = thr.get("nr_elements")
            sample = thr.get("sample")
            if nr_elements is None or sample is None:
                continue

            times: Sequence[float] = sample.get("times", [])
            iters: Sequence[float] = sample.get("iters", [])

            if len(times) != len(iters) or not times:
                raise Exception("Sample sizes do not match!")

            # Time per iteration (ns) for each sample
            times_per_iter = [
                t / it for t, it in zip(times, iters)
            ]

            # elements per second for each sample:
            # E * iters[i] / (times[i] ns) * 1e9
            throughputs = [
                nr_elements * it / t * 1e9
                for t, it in zip(times, iters)
            ]

            sample["times_per_iter"] = times_per_iter
            sample["throughputs"] = throughputs
            sample["nr_samples"] = len(throughputs)


def _get_dfs_by_workload(raw_results: dict) -> Dict[str, dict]:
    """
    Returns a dict:
      {
        config_key: {
          "df": pandas.DataFrame (per-strategy metrics, indexed by strategy),
          "raw": original raw_results[config_key] (strategies dict)
        },
        ...
      }
    Each entry in raw_results[config_key][strategy] is expected to have:
      - "memory"
      - "throughput"
      - "estimates"
      - "sample"
      - "tukey"
    """

    rows = []

    for config_key, strategies in raw_results.items():
        for strat_name, strat_data in strategies.items():
            mem_dict = strat_data["memory"]
            thr_metrics = strat_data["throughput"]
            estimates = thr_metrics["estimates"]  # new: nested estimates dict
            # sample = thr_metrics["sample"]      # available if you need it later
            # tukey = thr_metrics["tukey"]

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
                "time_mean_ns": estimates["mean"]["point_estimate"],
                "time_median_ns": estimates["median"]["point_estimate"],
                "time_std_dev_ns": estimates["std_dev"]["point_estimate"],
            }
            rows.append(row)

    df_all = pd.DataFrame(rows)
    df_all.set_index(["config", "strategy"], inplace=True)

    configs = df_all.index.get_level_values("config").unique()

    # Build per-workload structure that keeps both df and raw
    per_workload = {}

    for cfg in configs:
        df_cfg = df_all.xs(cfg, level="config").copy()
        df_cfg["thr_mean_elem_per_s"] = df_cfg["nr_elements"] / (df_cfg["time_mean_ns"] * 1e-9)

        per_workload[cfg] = {
            "df": df_cfg,
            "raw": raw_results[cfg],
        }

    # Global baseline across all configs/strategies
    baseline = max(
        per_workload[cfg]["df"]["thr_mean_elem_per_s"].max()
        for cfg in configs
    )

    for cfg in configs:
        df_cfg = per_workload[cfg]["df"]
        df_cfg["thr_mean_elem_rel"] = df_cfg["thr_mean_elem_per_s"] / baseline

    return per_workload


def get_results(path: Path) -> Dict[str, dict]:
    raw_results = _load_raw_results(path)
    add_sample_throughputs(raw_results)
    decorated = _get_dfs_by_workload(raw_results)
    return decorated
