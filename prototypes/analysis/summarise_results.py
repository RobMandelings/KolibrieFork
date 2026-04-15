from compare_strats_workloads_overview import build_overview_from_dfs, plot_overview

from results_parser import get_results
from sorting import sort_configs, to_labeled_dataframe_dict


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
