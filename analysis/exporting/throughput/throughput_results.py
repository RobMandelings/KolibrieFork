from pathlib import Path

import pandas as pd

from exporting.throughput.sample_plotting import plot_sample_with_outliers


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


def generate_throughput_results(workload_dir, strategy_name, strat_data):
    strat_dir = Path(workload_dir) / "throughput" / strategy_name / "results"
    strat_dir.mkdir(parents=True, exist_ok=True)

    sample = strat_data["throughput"]["sample"]

    estimates_to_csv(
        strat_data["throughput"]["estimates"],
        strat_dir / "estimates_summary.csv",
    )
    plot_sample_with_outliers(
        sample,
        "time",
        "Sample times",
        path=strat_dir / "samples_time.png",
    )
    plot_sample_with_outliers(
        sample,
        "throughput",
        "Sample throughputs",
        path=strat_dir / "samples_throughput.png",
    )
