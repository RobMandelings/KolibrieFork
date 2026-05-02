import copy
import json
import statistics
from pathlib import Path
from typing import Dict, Sequence, Any

import mem_results as mem_res_module
import pandas as pd

from parsing import throughput_results
from workload_keys import make_label_from_key


def load_workload_jsons(path: Path = None) -> Dict[str, Any]:
    """
    Load workload.json for all workloads in the given directory.
    Returns: {workload_name: workload_json_object}
    """
    all_results: Dict[str, Any] = {}

    for workload_dir in path.iterdir():
        if not workload_dir.is_dir() or workload_dir.name == "overviews":
            continue

        workload_name = workload_dir.name
        workload_json_path = workload_dir / "workload.json"

        if not workload_json_path.is_file():
            continue

        with open(workload_json_path, "r", encoding="utf-8") as f:
            all_results[workload_name] = json.load(f)

    return all_results


def load_results(path: Path) -> dict:
    strats = mem_res_module.load_mem_results(path)
    thr_results = throughput_results.load_results(path)
    workload_json_results = load_workload_jsons(path)
    results = {}

    for workload_name, strat_results in strats.items():
        results[workload_name] = {
            "strategies": {},
            "workload": workload_json_results[workload_name]
        }

        if workload_name not in strats or workload_name not in thr_results:
            raise Exception(f"{workload_name}: not present in both throughput and mem results")

        for strat_name, mem_result in strat_results.items():
            # pick the matching throughput result
            thr_result = thr_results[workload_name][strat_name]

            results[workload_name]["strategies"][strat_name] = {
                "memory": mem_result,
                "throughput": thr_result,
            }

    return results


def add_sample_information(result: Dict[str, Dict[str, Dict[str, Any]]]) -> None:
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
                raise Exception("Did not find throughput")

            nr_elements = thr.get("nr_elements")
            sample = thr.get("sample")
            if nr_elements is None or sample is None:
                raise Exception("nr elements and sample is not found")

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


def add_throughput_estimates(result: dict):
    for workload_key, workload_data in result.items():
        nr_events = workload_data["workload"]["nr_events"]
        strategies = workload_data["strategies"]
        for strat_name, strat_data in strategies.items():
            thr = strat_data.get("throughput")
            if not thr:
                raise Exception("Did not find throughput")

            nr_elements = thr.get("nr_elements")
            sample = thr.get("sample")
            estimates = thr.get("estimates")
            if nr_elements is None or sample is None:
                raise Exception("nr elements and sample is not found")

            throughputs = sample["throughputs"]
            mean_ns = estimates["mean"]["point_estimate"]
            median_ns = estimates["median"]["point_estimate"]

            estimates["thr_mean"] = nr_events / (mean_ns * 1e-9)
            estimates["thr_median"] = nr_events / (median_ns * 1e-9)

            estimates["thr_std_dev"] = statistics.stdev(throughputs)
            estimates["thr_min"] = min(throughputs)
            estimates["thr_max"] = max(throughputs)


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

    for cfg in configs:
        df_cfg = df_all.xs(cfg, level="config").copy()
        df_cfg["thr_mean_elem_per_s"] = df_cfg["nr_elements"] / (df_cfg["time_mean_ns"] * 1e-9)
        df_cfg["thr_median_elem_per_s"] = df_cfg["nr_elements"] / (df_cfg["time_median_ns"] * 1e-9)

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
    add_sample_information(results)
    add_throughput_estimates(results)
    add_throughput_df(results)
    add_labels_to_workloads(results)
    return results
