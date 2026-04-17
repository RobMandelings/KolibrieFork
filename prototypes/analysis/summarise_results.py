from __future__ import annotations

from pathlib import Path
from typing import Dict, Any

import numpy as np
import pandas as pd
from sample_plotting import plot_sample, plot_sample_with_outliers, plot_samples_grouped
from results_parser import get_results
from sorting import sort_configs
from typing import Mapping, Sequence


def get_dfs_by_workload(per_workload: Dict[str, dict]) -> Dict[str, pd.DataFrame]:
    """
    Given a dict like:
      { workload_key: {"df": DataFrame, "raw": ...}, ... }
    return:
      { workload_key: DataFrame, ... }
    """
    return {
        workload: entry["df"].copy()
        for workload, entry in per_workload.items()
    }


def mean_time_from_sample(sample: Mapping) -> float:
    """
    Compute the simple mean per-iteration time (in the same units as 'times',
    typically nanoseconds) from a Criterion sample.json-like structure:

        {
          "sampling_mode": "Flat",
          "iters": [i0, i1, ...],
          "times": [t0, t1, ...]
        }

    Returns:
        float: mean time per iteration.
    """
    iters: Sequence[float] = sample["iters"]
    times: Sequence[float] = sample["times"]

    if len(iters) != len(times) or not iters:
        raise ValueError("iters and times must be non-empty and of equal length")

    # Per-iteration times for each sample
    per_iter_times = [t / i for t, i in zip(times, iters)]

    # Simple arithmetic mean
    mean = sum(per_iter_times) / len(per_iter_times)
    return mean


def build_samples_by_strategy(
        per_workload: Dict[str, Dict[str, Any]]
) -> Dict[str, Dict[str, dict]]:
    """
    Given a structure like:

        per_workload = {
            workload_key: {
                "df":  DataFrame,
                "raw": {
                    strategy_name: {
                        "memory": {...},
                        "throughput": {
                            "nr_elements": ...,
                            "estimates": {...},
                            "sample": {
                                "sampling_mode": ...,
                                "iters": [...],
                                "times": [...],
                                # optionally "throughputs", "tukey", ...
                            },
                        },
                    },
                    ...
                },
            },
            ...
        }

    return:

        {
            workload_key: {
                strategy_name: sample_dict,  # the dict under throughput["sample"]
                ...
            },
            ...
        }
    """

    samples_by_workload: Dict[str, Dict[str, dict]] = {}

    for workload_key, workload_entry in per_workload.items():
        raw = workload_entry.get("raw", {})
        samples_for_workload: Dict[str, dict] = {}

        for strat_name, strat_data in raw.items():
            thr = strat_data.get("throughput")
            if not thr or "sample" not in thr:
                continue
            samples_for_workload[strat_name] = thr["sample"]

        if samples_for_workload:
            samples_by_workload[workload_key] = samples_for_workload

    return samples_by_workload


from pathlib import Path
from typing import Dict, Any, Callable


def throughput_elements_per_second(n_elements: int, time_ns: float) -> float:
    return n_elements * 1_000_000_000.0 / time_ns


def estimates_to_csv(estimates, output_path):
    print(f"Creating estimates to csv: {output_path}")
    rows = []

    for name in ["mean", "median", "median_abs_dev", "std_dev"]:
        est = estimates.get(name)
        if est is None:
            continue

        ci = est["confidence_interval"]
        rows.append({
            "statistic": name,
            "point_estimate": est["point_estimate"],
            "confidence_interval": (
                f"[{ci['lower_bound']}, {ci['upper_bound']}] "
                f"@ {ci['confidence_level']}"
            ),
            "standard_error": est["standard_error"],
        })

    df = pd.DataFrame(rows).set_index("statistic")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(output_path, index=True)


def workload_summary_to_csv(raw: dict, output_path):
    """
    raw: mapping from strategy_name -> strat_data as in your loop:
         strat_data["throughput"]["mean"]["point_estimate"] etc.
    n_elements: number of elements processed per run (used for throughput conversion).
    output_path: pathlib.Path or str to the summary CSV.
    """
    print(f"Summarising workloads to csv: {output_path}")
    rows = []

    for strategy_name, strat_data in raw.items():
        thr = strat_data["throughput"]
        nr_elements = thr["nr_elements"]
        mean_time = thr["estimates"]["mean"]["point_estimate"]  # ns
        median_time = thr["estimates"]["median"]["point_estimate"]  # ns

        mean_throughput = throughput_elements_per_second(nr_elements, mean_time)
        median_throughput = throughput_elements_per_second(nr_elements, median_time)

        rows.append({
            "strategy": strategy_name,
            "mean_time_ns": mean_time,
            "median_time_ns": median_time,
            "mean_throughput_eps": mean_throughput,
            "median_throughput_eps": median_throughput,
        })

    df = pd.DataFrame(rows).set_index("strategy")

    baseline = "clone"
    if baseline in df.index:
        base_mean = df.loc[baseline, "mean_throughput_eps"]
        base_median = df.loc[baseline, "median_throughput_eps"]

        # Ratios: >1 means faster than clone, <1 means slower
        df["speedup_vs_clone_mean"] = df["mean_throughput_eps"] / base_mean
        df["speedup_vs_clone_median"] = df["median_throughput_eps"] / base_median

    output_path.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(output_path, index=True)


def walk_workloads_and_strategies(
        per_workload: Dict[str, Dict[str, Any]],
        analysis_path: Path,
) -> None:
    """
    Iterate over each workload and its strategies.

    For each (workload_key, strategy_name), compute a path:
        analysis_path / workload_key / strategy_name

    and call a placeholder where you can later save figures, CSVs, etc.

    Structure assumed:
        per_workload[workload_key]["raw"][strategy_name] -> strat_data
    """
    for workload_key, entry in per_workload.items():
        raw = entry.get("raw", {})
        workload_dir = analysis_path / workload_key
        workload_output_dir = workload_dir / "overviews"
        workload_summary_to_csv(raw, workload_output_dir / "estimates.csv")
        plot_samples_grouped(raw, mode="throughput", path=str(workload_output_dir / "samples_throughput_grouped.png"))

        for strategy_name, strat_data in raw.items():
            # Build path: <analysis_path>/<workload>/throughput/<strategy>/
            strat_dir = workload_dir / "throughput" / strategy_name / "results"
            strat_dir.mkdir(parents=True, exist_ok=True)

            sample = strat_data["throughput"]["sample"]
            estimates_to_csv(strat_data["throughput"]["estimates"], strat_dir / "estimates_summary.csv")
            plot_sample_with_outliers(sample, "time", "Sample times", path=strat_dir / "samples_time.png")
            plot_sample_with_outliers(sample, "throughput", "Sample throughputs", path=strat_dir / "samples_throughput"
                                                                                                   ".png")


def main(analysis_path: Path):
    results = get_results(analysis_path)
    sorted_by_size_then_slide = sort_configs(results, "size", reverse=False)
    t = build_samples_by_strategy(sorted_by_size_then_slide)
    sample_test = list(t.values())[0]
    walk_workloads_and_strategies(results, analysis_path)


if __name__ == "__main__":
    main(Path("evaluation/15_04-2"))
