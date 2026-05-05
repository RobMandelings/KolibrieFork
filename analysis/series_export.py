import pandas as pd
from pathlib import Path


def result_to_dataframe(result_dict):
    """
    Long format:
    columns: x_label, strategy, value
    """
    records = []

    for strategy, workloads in result_dict.items():
        for workload_key, metrics in workloads.items():
            x_label = metrics.get("x_label")
            value = metrics.get("value")
            if x_label is None:
                continue
            records.append(
                {
                    "x_label": x_label,
                    "strategy": strategy,
                    "value": value,
                }
            )

    return pd.DataFrame(records)


def export_result_to_csv(result_dict, csv_path: str):
    """
    Convert the nested 'result' dict to a DataFrame and write it to CSV.
    """
    csv_path = Path(csv_path)
    csv_path.parent.mkdir(parents=True, exist_ok=True)

    df = result_to_dataframe(result_dict)
    df.to_csv(csv_path, index=False)
    return df
