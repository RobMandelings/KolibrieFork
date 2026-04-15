from dataclasses import dataclass

import pandas as pd


@dataclass
class LabeledDataFrame:
    label: str
    dataframe: pd.DataFrame


def parse_config_key(key: str) -> dict:
    return {
        part.split("=")[0]: int(part.split("=")[1])
        for part in key.split(",")
    }


def sort_configs(config_dict, *fields, reverse=False):
    """
    Return a new dict sorted by the given config fields.

    Example:
        sort_configs(dfs_by_config, "size")
        sort_configs(dfs_by_config, "windows", "size")
        sort_configs(dfs_by_config, "events", reverse=True)
    """
    return dict(
        sorted(
            config_dict.items(),
            key=lambda item: tuple(parse_config_key(item[0])[field] for field in fields),
            reverse=reverse,
        )
    )


def sort_by_size(config_dict, reverse=False):
    return sort_configs(config_dict, "size", reverse=reverse)


def sort_by_slide(config_dict, reverse=False):
    return sort_configs(config_dict, "slide", reverse=reverse)


def sort_by_windows(config_dict, reverse=False):
    return sort_configs(config_dict, "windows", reverse=reverse)


def sort_by_events(config_dict, reverse=False):
    return sort_configs(config_dict, "events", reverse=reverse)


def make_label_from_key(key: str) -> str:
    parts = parse_config_key(key)
    # Use whatever label format you want; this matches your "1,size,slide,events" idea
    return f"{parts['windows']},{parts['size']},{parts['slide']},{parts['events']}"


def to_labeled_dataframe_dict(config_dict):
    """
    Takes a dict {config_key: df} and returns
    {config_key: LabeledDataFrame(label=..., dataframe=df)}.
    The input order is preserved.
    """
    return {
        key: LabeledDataFrame(
            label=make_label_from_key(key),
            dataframe=df,
        )
        for key, df in config_dict.items()
    }