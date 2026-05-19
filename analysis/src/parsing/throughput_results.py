import json
from pathlib import Path

SKIP_NAMES = {"base", "new", "change", "report"}


def load_results(path: Path = None):
    results = {}

    for estimates_path in path.rglob("estimates.json"):

        benchmark_path = estimates_path.parent / "benchmark.json"
        if not benchmark_path.is_file():
            continue

        # TODO perhaps it is better to use a workload_name from the workload.json (if there is time)
        workload_name = estimates_path.parent.parent.parent.name

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

        value_str = benchmark["value_str"]

        # New: store subfields instead of flattening estimates
        entry = {
            "estimates": estimates,
            "sample": sample,
            "tukey": tukey,
            "nr_elements": benchmark["throughput"]["Elements"],
        }

        if not workload_name in results:
            results[workload_name] = {}
        results[workload_name][value_str] = entry

    return results


if __name__ == "__main__":
    results = load_results()
