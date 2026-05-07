from pathlib import Path

import pandas as pd

from arg_parser import parse_args
from overview_plotters import make_default_overview_plotters


def generate_plots(analysis_path: Path):
    df = pd.read_csv(analysis_path / "csv" / "summary.csv")
    plotters = make_default_overview_plotters()
    for plotter in plotters:
        plotter(df, analysis_path)


def main() -> None:
    args = parse_args()
    analysis_path = Path(args.target).resolve()
    print(f"Using analysis path: {analysis_path}")
    generate_plots(analysis_path)


if __name__ == "__main__":
    main()
