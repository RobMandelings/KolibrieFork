import copy
from pathlib import Path
from typing import Dict, Sequence, Any

import mem_results as mem_res_module
import pandas as pd

from parsing import throughput_results
from workload_keys import make_label_from_key


def load_results(path: Path) -> dict:
    strats = mem_res_module.load_mem_results(path)
    thr_results = throughput_results.load_results(path)
    combined_mem_throughput = {}

    for workload_name, strat_results in strats.items():
        combined_mem_throughput[workload_name] = {
            "strategies": {}
        }

        if workload_name not in strats or workload_name not in thr_results:
            print(f"Ignoring workload {workload_name}: not present in both throughput and mem results")
            continue

        for strat_name, mem_result in strat_results.items():
            # pick the matching throughput result
            thr_result = thr_results[workload_name][strat_name]

            combined_mem_throughput[workload_name]["strategies"][strat_name] = {
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

    for workload_key, workload_data in result.items():
        strategies = workload_data["strategies"]
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


def add_throughput_df(results: dict) -> None:
    """
    Returns a dict shaped like the original raw_results, but with an added
    aggregated dataframe under each config's 'throughput' key:

      {
        config_key: {
          ... original strategies ...,
          "throughput": {
            "dataframe": pandas.DataFrame
          }
        },
        ...
      }

    The dataframe contains one row per strategy and is indexed by strategy.
    """

    rows = []

    for workload_key, workload_data in results.items():
        strategies = workload_data["strategies"]
        for strat_name, strat_data in strategies.items():
            thr_metrics = strat_data["throughput"]
            estimates = thr_metrics["estimates"]

            row = {
                "config": workload_key,
                "strategy": strat_name,
                "nr_elements": thr_metrics["nr_elements"],
                "time_mean_ns": estimates["mean"]["point_estimate"],
                "time_median_ns": estimates["median"]["point_estimate"],
                "time_std_dev_ns": estimates["std_dev"]["point_estimate"],
            }
            rows.append(row)

    df_all = pd.DataFrame(rows)
    df_all.set_index(["config", "strategy"], inplace=True)

    configs = df_all.index.get_level_values("config").unique()

    per_cfg_dfs = {}
    for cfg in configs:
        df_cfg = df_all.xs(cfg, level="config").copy()
        df_cfg["thr_mean_elem_per_s"] = df_cfg["nr_elements"] / (df_cfg["time_mean_ns"] * 1e-9)
        per_cfg_dfs[cfg] = df_cfg

    baseline = max(
        df_cfg["thr_mean_elem_per_s"].max()
        for df_cfg in per_cfg_dfs.values()
    )

    for cfg, df_cfg in per_cfg_dfs.items():
        df_cfg["thr_mean_elem_rel"] = df_cfg["thr_mean_elem_per_s"] / baseline

        results[cfg]["throughput_df"] = df_cfg


def add_labels_to_workloads(result: dict) -> None:
    """
    Adds a 'label' field under each top-level workload entry.

    Example:
        result["windows=1,size=8,slide=1,events=50000"]["label"] = "1,8,1,50000"

    Returns the same dict for convenience.
    """
    for workload_key, workload_data in result.items():
        workload_data["label"] = make_label_from_key(workload_key)


def get_results(path: Path) -> Dict[str, dict]:
    results = load_results(path)
    add_sample_throughputs(results)
    add_throughput_df(results)
    add_labels_to_workloads(results)
    return results
