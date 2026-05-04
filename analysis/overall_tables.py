from __future__ import annotations

from typing import Callable, Any

import pandas as pd


def dict_to_metric_table(
    results: dict,
    value_fn: Callable[[dict, dict], Any],
    output_path: str | None = None,
) -> pd.DataFrame:

    table_data = {}

    for workload, workload_data in results.items():
        strategies = workload_data["strategies"]
        table_data[workload] = {}

        for strategy, strategy_data in strategies.items():
            value = value_fn(workload_data, strategy_data)
            table_data[workload][strategy] = value

    df = pd.DataFrame(table_data)
    df.index.name = "strategy"
    df.columns.name = "workload"

    if output_path is not None:
        df.to_csv(output_path)

    return df