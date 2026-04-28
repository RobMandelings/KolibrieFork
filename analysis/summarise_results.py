from __future__ import annotations

from typing import Any
from typing import Dict
from typing import Mapping, Sequence

import pandas as pd
from matplotlib import pyplot as plt

from constants import STRATEGY_COLORS, STRATEGY_MARKERS
from exporting.compare_strats_workloads_overview import build_and_export_overviews
from exporting.throughput.sample_plotting import plot_samples_grouped
from exporting.throughput.throughput_results import generate_throughput_results
from exporting.workload_csv import workload_summary_to_csv
from organising import sorting
from organising.sorting import LabeledDataFrame, sort_configs
from parsing.dhat_parser import parse_dhat
from parsing.results_parser import get_results
from workload_keys import make_label_from_key


def get_dfs_by_workload(per_workload: Dict[str, dict]) -> Dict[str, pd.DataFrame]:
    """
    Given a dict like:
      { workload_key: {"df": DataFrame, "raw": ...}, ... }
    return:
      { workload_key: DataFrame, ... }
    """
    return {
        workload: entry["df"].copy()
        for workload, entry in per_workload.items()
    }


def mean_time_from_sample(sample: Mapping) -> float:
    """
    Compute the simple mean per-iteration time (in the same units as 'times',
    typically nanoseconds) from a Criterion sample.json-like structure:

        {
          "sampling_mode": "Flat",
          "iters": [i0, i1, ...],
          "times": [t0, t1, ...]
        }

    Returns:
        float: mean time per iteration.
    """
    iters: Sequence[float] = sample["iters"]
    times: Sequence[float] = sample["times"]

    if len(iters) != len(times) or not iters:
        raise ValueError("iters and times must be non-empty and of equal length")

    # Per-iteration times for each sample
    per_iter_times = [t / i for t, i in zip(times, iters)]

    # Simple arithmetic mean
    mean = sum(per_iter_times) / len(per_iter_times)
    return mean


def build_samples_by_strategy(
        per_workload: Dict[str, Dict[str, Any]]
) -> Dict[str, Dict[str, dict]]:
    """
    Given a structure like:

        per_workload = {
            workload_key: {
                "df":  DataFrame,
                "raw": {
                    strategy_name: {
                        "memory": {...},
                        "throughput": {
                            "nr_elements": ...,
                            "estimates": {...},
                            "sample": {
                                "sampling_mode": ...,
                                "iters": [...],
                                "times": [...],
                                # optionally "throughputs", "tukey", ...
                            },
                        },
                    },
                    ...
                },
            },
            ...
        }

    return:

        {
            workload_key: {
                strategy_name: sample_dict,  # the dict under throughput["sample"]
                ...
            },
            ...
        }
    """

    samples_by_workload: Dict[str, Dict[str, dict]] = {}

    for workload_key, workload_entry in per_workload.items():
        raw = workload_entry.get("raw", {})
        samples_for_workload: Dict[str, dict] = {}

        for strat_name, strat_data in raw.items():
            thr = strat_data.get("throughput")
            if not thr or "sample" not in thr:
                continue
            samples_for_workload[strat_name] = thr["sample"]

        if samples_for_workload:
            samples_by_workload[workload_key] = samples_for_workload

    return samples_by_workload


from pathlib import Path
from typing import Dict, Any


def generate_memory_results(workload_dir, strategy_name):
    strat_dir = Path(workload_dir) / "memory"

    dhat_summary = parse_dhat(str(strat_dir / f"{strategy_name}.json"))
    output_path = strat_dir / "results" / f"{strategy_name}_overview.csv"
    output_path.parent.mkdir(parents=True, exist_ok=True)
    dhat_summary.to_csv(output_path, index=True)


PROPS_TO_PLOT = [
    "total_bytes",
    "total_blocks",
    "total_bytes_pct",
    "total_blocks_pct",
]

PATHS_TO_PLOT = [
    "TOTAL",
    "vector_clone_from_window_closed",
    "element_clone_from_window_closed"
]


def plot_all_memory_properties(per_workload: dict, output_dir: Path):
    for path in PATHS_TO_PLOT:
        for prop in PROPS_TO_PLOT:
            df = build_strategy_workload_table(
                results=per_workload,
                path=path,
                prop=prop,
            )

            plot_strategy_workload_table(
                df,
                title=f"{prop} for {path}",
                ylabel=prop,
                path=output_dir / f"{prop}_for_{path}.png"
            )


def walk_workloads_and_strategies(
        per_workload: Dict[str, Dict[str, Any]],
        analysis_path: Path,
) -> None:
    """
    Iterate over each workload and its strategies.

    For each (workload_key, strategy_name), compute a path:
        analysis_path / workload_key / strategy_name

    and call a placeholder where you can later save figures, CSVs, etc.

    Structure assumed:
        per_workload[workload_key]["raw"][strategy_name] -> strat_data
    """

    labeled_dfs = to_labeled_dataframe_list(get_throughput_dfs_by_workload(per_workload))
    build_and_export_overviews(labeled_dfs, analysis_path / "overviews")

    plot_all_memory_properties(per_workload, analysis_path / "overviews" / "png")

    for workload_key, entry in per_workload.items():
        workload_dir = analysis_path / workload_key
        workload_output_dir = workload_dir / "overviews"
        workload_summary_to_csv(entry, workload_output_dir / "estimates.csv")
        plot_samples_grouped(entry, mode="throughput", path=str(workload_output_dir / "samples_throughput_grouped.png"))

        for strategy_name, strat_data in entry["strategies"].items():
            generate_throughput_results(workload_dir, strategy_name, strat_data)
            generate_memory_results(workload_dir, strategy_name)


def extract_dfs(result: dict) -> dict:
    """
    Turn:
      { key: {"df": DataFrame, "raw": ...}, ... }
    into:
      { key: DataFrame, ... }
    """
    return {key: value["df"] for key, value in result.items() if "df" in value}


def build_strategy_workload_table(
        results: Dict[str, Any],
        path: str,
        prop: str,
        memory_key: str = "memory",
) -> pd.DataFrame:
    """
    Build a single table with:
      - rows    = strategies
      - columns = workloads
      - values  = memory_df.loc[path, prop]

    Example lookup used:
      results[workload][strategy][memory_key].loc[path, prop]

    Parameters
    ----------
    results : dict
        Nested dict like:
        {
          workload_key: {
            strategy_name: {
              "memory": pd.DataFrame,
              "throughput": ...
            },
            ...
          },
          ...
        }

    path : str
        Row label inside the memory dataframe,
        e.g. "vector_clone_from_window_close"

    prop : str
        Column name inside the memory dataframe,
        e.g. "total_bytes"

    memory_key : str
        Usually "memory".

    Returns
    -------
    pd.DataFrame
        Index = strategies
        Columns = workloads
    """

    table = {}

    for workload_key, workload_data in results.items():
        strategies = workload_data["strategies"]
        workload_label = workload_data.get("label")

        for strategy, strategy_data in strategies.items():
            if strategy == "label":
                continue

            if not isinstance(strategy_data, dict):
                continue

            mem_df = strategy_data.get(memory_key)
            if mem_df is None or not isinstance(mem_df, pd.DataFrame):
                continue

            value = pd.NA
            if path in mem_df.index and prop in mem_df.columns:
                value = mem_df.loc[path, prop]

            if strategy not in table:
                table[strategy] = {}

            table[strategy][workload_label] = value

    df = pd.DataFrame.from_dict(table, orient="index")
    df.index.name = "strategy"
    return df


def plot_strategy_workload_table(
        df: pd.DataFrame,
        *,
        title: str | None = None,
        ylabel: str | None = None,
        figsize: tuple[int, int] = (12, 6),
        marker: str = "o",
        linewidth: float = 2.0,
        sort_columns: bool = False,
        ax=None,
        path: Path = None
):
    """
    Plot a DataFrame where:
      - df.index   = strategies
      - df.columns = workloads
      - df.values  = metric values (possibly containing pd.NA)

    Each strategy is plotted as one line over the workloads.
    """

    plot_df = df.copy()

    if sort_columns:
        plot_df = plot_df.reindex(sorted(plot_df.columns), axis=1)

    # Make sure <NA> / object values become numeric NaN where needed
    plot_df = plot_df.apply(pd.to_numeric, errors="coerce")

    if ax is None:
        fig, ax = plt.subplots(figsize=figsize)
    else:
        fig = ax.figure

    x = list(range(len(plot_df.columns)))
    xlabels = [str(c) for c in plot_df.columns]

    for strategy in plot_df.index:
        y = plot_df.loc[strategy].to_numpy(dtype=float)

        # Skip fully empty rows
        if pd.isna(y).all():
            continue

        ax.plot(
            x,
            y,
            color=STRATEGY_COLORS.get(strategy, "gray"),
            marker=STRATEGY_MARKERS.get(strategy, "o"),
            linewidth=linewidth,
            label=strategy,
        )

    ax.set_xticks(x)
    ax.set_xticklabels(xlabels, rotation=45, ha="right")
    ax.set_xlabel("Workload")

    if ylabel is not None:
        ax.set_ylabel(ylabel)

    if title is not None:
        ax.set_title(title)

    ax.grid(True, axis="y", alpha=0.3)
    ax.legend(title="Strategy", loc="best")
    fig.tight_layout()

    if path is None:
        plt.show()
    else:
        fig.savefig(path, dpi=300, bbox_inches="tight")
        plt.close()


def to_labeled_dataframe_list(config_dict):
    """
    Takes a dict {config_key: df} and returns
    {config_key: LabeledDataFrame(label=..., dataframe=df)}.
    The input order is preserved.
    """
    return list({
                    key: LabeledDataFrame(
                        label=make_label_from_key(key),
                        dataframe=df,
                    )
                    for key, df in config_dict.items()
                }.values())


def get_throughput_dfs_by_workload(results: Dict[str, Any]) -> Dict[str, pd.DataFrame]:
    """
    Return a dict mapping each workload key directly to its throughput_df.

    Example output:
        {
            "windows=1,size=8,slide=1,events=50000": <DataFrame>,
            "windows=1,size=2,slide=1,events=50000": <DataFrame>,
            ...
        }
    """
    out = {}

    for workload, workload_data in results.items():
        throughput_df = workload_data.get("throughput_df")
        if isinstance(throughput_df, pd.DataFrame):
            out[workload] = throughput_df

    return out


def main(analysis_path: Path):
    results = get_results(analysis_path)
    sorted_by_size_then_slide = sort_configs(results, "size", reverse=False)

    labeled_dfs = to_labeled_dataframe_list(get_throughput_dfs_by_workload(sorted_by_size_then_slide))
    build_and_export_overviews(labeled_dfs, analysis_path / "overviews")

    walk_workloads_and_strategies(sorted_by_size_then_slide, analysis_path)


def main_pipeline(analysis_path: Path):
    results = get_results(analysis_path)
    results = sorting.sort_by_size(results, reverse=False)
    # results = sorting.sort_by_slide(results, reverse=False)

    walk_workloads_and_strategies(results, analysis_path)


if __name__ == "__main__":
    main_pipeline(Path("evaluation/size_vary1_32_bytes"))
