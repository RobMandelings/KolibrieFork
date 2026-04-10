import json
from pathlib import Path

ROOT = Path("evaluation")

SKIP_NAMES = {"base", "new", "change", "report"}


def load_estimates(dir_suffix: str = None):
    results = {}

    if dir_suffix:
        root = ROOT / dir_suffix
    else:
        root = ROOT

    for estimates_path in root.rglob("estimates.json"):

        benchmark_path = estimates_path.parent / "benchmark.json"
        if not benchmark_path.is_file():
            continue

        with estimates_path.open() as f:
            estimates = json.load(f)
        with benchmark_path.open() as f:
            benchmark = json.load(f)

        group_id = benchmark["group_id"]
        value_str = benchmark["value_str"]

        entry = dict(estimates)
        entry["nr_elements"] = benchmark["throughput"]["Elements"]

        if not group_id in results:
            results[group_id] = {}
        results[group_id][value_str] = entry

    return results


if __name__ == "__main__":
    results = load_estimates()
    print("Hi")
