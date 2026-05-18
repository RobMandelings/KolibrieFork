import argparse


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