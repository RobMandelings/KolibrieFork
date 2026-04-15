import re

from compare_strats_workloads_overview import build_overview_from_dfs, plot_overview

from results_parser import get_results
from sorting import sort_configs, to_labeled_dataframe_dict


def parse_size(key: str) -> int:
    """Extract integer window size from 'size=1,slide=1,offset=0'."""
    m = re.search(r"size\s*=\s*(\d+)", key)
    if not m:
        raise ValueError(f"Cannot parse size from key: {key}")
    return int(m.group(1))


def filter_by_nr_elements(results, n):
    return {
        name: data
        for name, data in results.items()
        if "10000" in name
    }


def parse_workload_key(key: str) -> dict:
    parts = key.split("_")
    out = {}
    for part in parts:
        if "=" not in part:
            continue  # ignore pieces that can't be split
        k, v = part.split("=", 1)
        try:
            out[k] = int(v)
        except ValueError:
            out[k] = v
    return out


def filter_by_events(results: dict, target_events: int) -> dict:
    return {
        key: value
        for key, value in results.items()
        if parse_workload_key(key).get("events") == target_events
    }


def main():
    dfs_by_workload = get_results("15_04-2")
    sorted_by_size_then_slide = sort_configs(dfs_by_workload, "size", reverse=False)
    labeled = to_labeled_dataframe_dict(sorted_by_size_then_slide)

    as_values = labeled.values()

    overview_df = build_overview_from_dfs(as_values, "thr_mean_elem_rel")
    plot_overview(overview_df, "thr_mean_elem_rel", False)

    overview_df = build_overview_from_dfs(as_values, "mem_total_blocks")
    plot_overview(overview_df, "mem_total_blocks", False)

    overview_df = build_overview_from_dfs(as_values, "mem_total_bytes")
    plot_overview(overview_df, "mem_total_bytes", False)


if __name__ == "__main__":
    main()
