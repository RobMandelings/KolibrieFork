import argparse
from pathlib import Path

from arg_parser import parse_args
from parsing.results_parser import get_results
from series import workloads_to_dataframe, workloads_samples_to_dataframe


def generate_csvs(analysis_path: Path):
    csv_dir = analysis_path / "csv"
    csv_dir.mkdir(parents=True, exist_ok=True)
    print(f"Outputting to csv dir: {csv_dir}")
    workloads = get_results(analysis_path / "raw")
    df = workloads_to_dataframe(workloads)
    df.to_csv(csv_dir / "summary.csv", index=False)
    samples_df = workloads_samples_to_dataframe(workloads)
    samples_df.to_csv(csv_dir / "samples.csv", index=False)


def main() -> None:
    args = parse_args()
    analysis_path = Path(args.target).resolve()
    print(f"Using analysis path: {analysis_path}")

    generate_csvs(analysis_path)


if __name__ == "__main__":
    main()
