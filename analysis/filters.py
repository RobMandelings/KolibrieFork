from typing import List, Tuple, Any

import pandas as pd


def _parse_scalar(value: str) -> Any:
    lower = value.lower()

    if lower == "true":
        return True
    if lower == "false":
        return False
    if lower == "none" or lower == "null":
        return None

    try:
        return int(value)
    except ValueError:
        pass

    try:
        return float(value)
    except ValueError:
        pass

    return value


def apply_filters(df: pd.DataFrame, filters: List[Tuple[str, str, str]]) -> pd.DataFrame:
    filtered_df = df

    for column, op, raw_value in filters:
        if column not in filtered_df.columns:
            raise ValueError(
                f"Cannot filter on column {column!r}. "
                f"Available columns: {list(filtered_df.columns)}"
            )

        series = filtered_df[column]

        if op in {"in", "not in"}:
            values = [_parse_scalar(v.strip()) for v in raw_value.split(",")]
            mask = series.isin(values)
            if op == "not in":
                mask = ~mask
        else:
            value = _parse_scalar(raw_value)

            if op in {"=", "=="}:
                mask = series == value
            elif op == "!=":
                mask = series != value
            elif op == ">":
                mask = series > value
            elif op == ">=":
                mask = series >= value
            elif op == "<":
                mask = series < value
            elif op == "<=":
                mask = series <= value
            else:
                raise ValueError(f"Unsupported operator: {op!r}")

        filtered_df = filtered_df.loc[mask]

    return filtered_df
