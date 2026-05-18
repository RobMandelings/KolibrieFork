import argparse

ALLOWED_OPERATORS = {"=", "==", "!=", ">", ">=", "<", "<=", "in", "not in"}


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
    parser.add_argument(
        "--filter",
        "-f",
        dest="filters",
        action="append",
        nargs="+",
        metavar="FILTER",
        help=(
            "Filter rows before writing CSVs. "
            "Repeat this option to apply multiple filters. "
            "Examples: "
            "--filter bytes = 0 "
            "--filter threads >= 4 "
            "--filter engine in cqr,csr"
        ),
    )

    args = parser.parse_args()

    normalized_filters = []
    for raw_filter in args.filters or []:
        if len(raw_filter) < 3:
            parser.error(
                f"Invalid filter {' '.join(raw_filter)!r}. "
                "Expected: --filter COLUMN OP VALUE"
            )

        column = raw_filter[0]

        if raw_filter[1] == "not" and len(raw_filter) >= 4 and raw_filter[2] == "in":
            op = "not in"
            value = " ".join(raw_filter[3:])
        else:
            op = raw_filter[1]
            value = " ".join(raw_filter[2:])

        if op not in ALLOWED_OPERATORS:
            parser.error(
                f"Unsupported operator {op!r}. "
                f"Supported operators: {', '.join(sorted(ALLOWED_OPERATORS))}"
            )

        normalized_filters.append((column, op, value))

    args.filters = normalized_filters
    return args
