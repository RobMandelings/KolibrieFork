import pandas as pd


def throughput_elements_per_second(n_elements: int, time_ns: float) -> float:
    return n_elements * 1_000_000_000.0 / time_ns


def add_speedup_columns(df, baselines=("clone", "legacy")):
    """
    For each baseline name in `baselines` that exists in df.index,
    add mean/median speedup columns vs that baseline.
    """
    for baseline in baselines:
        if baseline not in df.index:
            continue

        base_mean = df.loc[baseline, "mean_throughput_eps"]
        base_median = df.loc[baseline, "median_throughput_eps"]

        df[f"speedup_vs_{baseline}_mean"] = df["mean_throughput_eps"] / base_mean
        df[f"speedup_vs_{baseline}_median"] = df["median_throughput_eps"] / base_median

    return df


def workload_summary_to_csv(workload: dict, output_path):
    """
    raw: mapping from strategy_name -> strat_data as in your loop:
         strat_data["throughput"]["mean"]["point_estimate"] etc.
    n_elements: number of elements processed per run (used for throughput conversion).
    output_path: pathlib.Path or str to the summary CSV.
    """
    print(f"Summarising workloads to csv: {output_path}")
    rows = []

    if len(workload["strategies"]) == 0:
        raise Exception(f"There are no strategies in workload {workload['label']}. Check directory.")

    for strategy_name, strat_data in workload["strategies"].items():

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
    df = add_speedup_columns(df)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(output_path, index=True)
