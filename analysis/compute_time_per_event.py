from pathlib import Path

import pandas as pd

from arg_parser import parse_args


def main() -> None:
    args = parse_args()
    analysis_path = Path(args.target).resolve()
    print(f"Using analysis path: {analysis_path}")

    # Read the CSV once
    df = pd.read_csv(analysis_path)
    df = df[df["reserve"] == 0]
    df["ns_per_event"] = df["sec_mean"] / df["nr_events"]

    df = df.sort_values("ns_per_event")
    # Print ns_per_event for each strategy
    for _, row in df.iterrows():
        print(f"Strategy: {row['strategy']}, ns_per_event: {row['ns_per_event']:.2f}")


if __name__ == "__main__":
    main()
