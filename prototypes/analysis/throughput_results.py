import json
from pathlib import Path

SKIP_NAMES = {"base", "new", "change", "report"}


def load_estimates(path: Path = None):
    results = {}

    for estimates_path in path.rglob("estimates.json"):

        benchmark_path = estimates_path.parent / "benchmark.json"
        if not benchmark_path.is_file():
            continue

        sample_path = estimates_path.parent / "sample.json"
        tukey_path = estimates_path.parent / "tukey.json"

        with estimates_path.open() as f:
            estimates = json.load(f)
        with benchmark_path.open() as f:
            benchmark = json.load(f)

        sample = None
        if sample_path.is_file():
            with sample_path.open() as f:
                sample = json.load(f)

        tukey = None
        if tukey_path.is_file():
            with tukey_path.open() as f:
                tukey = json.load(f)

        group_id = benchmark["group_id"]
        value_str = benchmark["value_str"]

        # New: store subfields instead of flattening estimates
        entry = {
            "estimates": estimates,
            "sample": sample,
            "tukey": tukey,
            "nr_elements": benchmark["throughput"]["Elements"],
        }

        if not group_id in results:
            results[group_id] = {}
        results[group_id][value_str] = entry

    return results


if __name__ == "__main__":
    results = load_estimates()
    print("Hi")
