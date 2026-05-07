import argparse
from pathlib import Path

from parsing.results_parser import get_results
from series import workloads_to_dataframe, workloads_samples_to_dataframe


def generate_csvs(analysis_path: Path):
    csv_dir = analysis_path / "csv"
    csv_dir.mkdir(parents=True, exist_ok=True)
    workloads = get_results(analysis_path / "raw")
    df = workloads_to_dataframe(workloads)
    df.to_csv(csv_dir / "summary.csv", index=False)
    samples_df = workloads_samples_to_dataframe(workloads)
    samples_df.to_csv(csv_dir / "samples.csv", index=False)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate CSV summaries from raw workload results"
    )
    parser.add_argument(
        "--target",
        "-t",
        required=True,
        help=(
            "Path to the analysis directory. "
            "The script will read from <target>/raw and write CSVs into <target>/csv."
        ),
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    analysis_path = Path(args.target).resolve()
    print(f"Using analysis path: {analysis_path}")

    generate_csvs(analysis_path)


if __name__ == "__main__":
    main()
