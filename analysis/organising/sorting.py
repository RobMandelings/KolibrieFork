from dataclasses import dataclass

import pandas as pd

from workload_keys import parse_config_key


@dataclass
class LabeledDataFrame:
    label: str
    dataframe: pd.DataFrame


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
