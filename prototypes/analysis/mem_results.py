#!/usr/bin/env python3
import json
from dataclasses import dataclass
from typing import Dict, List, Any
import pandas as pd
from pathlib import Path

ROOT = Path("evaluation")


@dataclass
class Metric:
    total_bytes: int  # sum of tb
    total_blocks: int  # sum of tbk
    t_gmax_bytes: int  # sum of gb (bytes at t-gmax)
    t_gmax_blocks: int  # sum of gbk


StrategyResult = Dict[str, Metric]


def load_dhat_file(path: str) -> Any:
    """Load a dhat-heap.json file."""
    with open(path, "r") as f:
        return json.load(f)


def extract_metrics_for_label(
        data: Dict[str, Any],
        substrings: List[str],
) -> Metric:
    """
    Aggregate metrics over all PPs whose backtrace contains any of the given substrings.

    - data: full dhat JSON (with keys 'pps' and 'ftbl')
    - substrings: list of substrings to search for in frame strings (OR semantics)
    """
    pps = data["pps"]
    ftbl = data["ftbl"]

    total_bytes = 0
    total_blocks = 0
    t_gmax_bytes = 0
    t_gmax_blocks = 0

    for pp in pps:
        # Build backtrace for this program point
        backtrace = [ftbl[i - 1] for i in pp["fs"]]

        # Special case: if [root] is mentioned, that means you must get the absolute root
        if substrings.__contains__("[root]"):
            if not len(backtrace) == 1 or not backtrace.__contains__("[root]"):
                continue

        # Does any frame contain one of the substrings?
        if not any(
                any(sub in frame for sub in substrings)
                for frame in backtrace
        ):
            continue

        # Sum metrics for this PP
        total_bytes += pp.get("tb", 0)
        total_blocks += pp.get("tbk", 0)
        t_gmax_bytes += pp.get("gb", 0)
        t_gmax_blocks += pp.get("gbk", 0)

    return Metric(
        total_bytes=total_bytes,
        total_blocks=total_blocks,
        t_gmax_bytes=t_gmax_bytes,
        t_gmax_blocks=t_gmax_blocks,
    )


def load_strategy_results_keyword_based(
        path: str,
        strategy_name: str,
        label_to_substrings: Dict[str, List[str]],
) -> StrategyResult:
    """
    Load one dhat JSON file and extract metrics for a set of labelled keyword patterns.

    label_to_substrings maps a logical label (e.g. "window_closed") to a list of
    substrings that, if present in any frame of a PP's backtrace, cause that PP
    to be counted for that label.
    """
    data = load_dhat_file(path)

    metrics: Dict[str, Metric] = {}

    total_bytes = sum(pp["tb"] for pp in data["pps"])
    total_blocks = sum(pp["tbk"] for pp in data["pps"])
    t_gmax_bytes = sum(pp["gb"] for pp in data["pps"])
    t_gmax_blocks = sum(pp["gbk"] for pp in data["pps"])

    # Root is always present here
    metrics["root"] = Metric(
        total_bytes=total_bytes,
        total_blocks=total_blocks,
        t_gmax_bytes=t_gmax_bytes,
        t_gmax_blocks=t_gmax_blocks,
    )

    if label_to_substrings:
        for label, substrings in label_to_substrings.items():
            metrics[label] = extract_metrics_for_label(data, substrings)
    return metrics


# ---------------------------------------------------------------------------

# Configure your strategies here: file paths + label -> list of substrings.
# You can add more labels or more substrings per label if needed.

# TODO make this flexible for any type of strategy
STRATEGY_CONFIG: Dict[str, dict] = {
    "clone": {
        "labels": {
            "window_closed": [
                "CloneStrategy as RSPPrototype::prototype::slide_strategy::WindowSnapshotStrategy>::window_closed",
                "CloneStrategy::window_closed",  # shorter backup
            ],
            "make_payload": [
                "RSPPrototype::prototype::helpers::make_payload (src/prototype/helpers.rs:13:9)"
            ],
        },
    },
    "refcount": {
        "labels": {
            "window_closed": [
                "window_closed",  # adjust to your actual refcount frame names
            ],
            "make_payload": [
                "RSPPrototype::prototype::helpers::make_payload (src/prototype/helpers.rs:13:9)"
            ],
        },
    },
    "arc": {
        "labels": {
            "window_closed": [
                "window_closed",  # adjust to your actual refcount frame names
            ],
            "make_payload": [
                "RSPPrototype::prototype::helpers::make_payload (src/prototype/helpers.rs:13:9)"
            ],
        },
    },
    "legacy": {
        "labels": {
        },
    },
    "expire": {
        "labels": {
            "make_payload": [
                "RSPPrototype::prototype::helpers::make_payload (src/prototype/helpers.rs:13:9)"
            ],
        },
    },
}

CONFIG_IDS: List[int] = [0, 1, 2, 3, 4]


def load_mem_results(dir_suffix: str = None) -> Dict[str, Dict[str, StrategyResult]]:
    """
    Load results for all workloads and strategies from mem_profiles/.
    Returns: {workload_name: {strategy_name: StrategyResult}}
    """
    all_results: Dict[str, Dict[str, StrategyResult]] = {}

    if dir_suffix:
        root = ROOT / dir_suffix
    else:
        root = ROOT

    for workload_dir in root.iterdir():
        if not workload_dir.is_dir():
            continue

        workload_name = workload_dir.name
        cfg_results: Dict[str, StrategyResult] = {}

        memory_dir = workload_dir / "memory"
        if not memory_dir.is_dir():
            continue

        # we expect files clone.json, expire.json, refcount.json
        for strat_name, cfg in STRATEGY_CONFIG.items():
            labels = cfg["labels"]
            path = memory_dir / f"{strat_name}.json"
            if not path.is_file():
                continue  # or raise if you want strictness

            cfg_results[strat_name] = load_strategy_results_keyword_based(
                str(path), strat_name, labels
            )

        if cfg_results:
            all_results[workload_name] = cfg_results

    return all_results


def results_to_dataframe(results) -> pd.DataFrame:
    rows = []
    for strat_name, strat_result in results.items():
        for label, m in strat_result.metrics.items():
            rows.append(
                {
                    "strategy": strat_name,
                    "label": label,
                    "total_bytes": m.total_bytes,
                    "total_blocks": m.total_blocks,
                    "t_gmax_bytes": m.t_gmax_bytes,
                    "t_gmax_blocks": m.t_gmax_blocks,
                }
            )
    return pd.DataFrame(rows)


if __name__ == "__main__":
    results = load_mem_results()
    print(results)
