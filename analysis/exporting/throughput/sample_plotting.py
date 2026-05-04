from __future__ import annotations

import statistics
from typing import Mapping, Literal, Optional, Iterable

from matplotlib import pyplot as plt

from constants import STRATEGY_COLORS, STRATEGY_MARKERS
from exporting.throughput.sample_outlier_detection import tukey_outliers_from_sample


def extract_samples_by_strategy(workload: dict) -> dict:
    return {
        strategy: data["throughput"]["sample"]
        for strategy, data in workload["strategies"].items()
        if "throughput" in data and "sample" in data["throughput"]
    }


def plot_samples_grouped(
        workload: dict,
        title: str | None = None,
        path: str = None,
) -> None:
    """
    Plot multiple Criterion samples (e.g., 'clone', 'expire', 'legacy', 'refcount')
    on the same axes.

    Parameters:
        samples: dict mapping strategy name -> sample dict
                 sample must contain:
                   - "times": list of total sample times in ns
                   - "iters": list of iterations per sample
                   - "throughputs": list of elements/sec (required if mode='throughput')
        mode: "time"        -> plot per-iteration time (ns) vs sample index
              "throughput"  -> plot throughput (elements/sec) vs sample index
        title: optional plot title
    """
    print(f"Plotting grouped samples: {path}")
    fig, ax = plt.subplots()
    samples = extract_samples_by_strategy(workload)

    ylabel = "Throughput (elements / s)"
    for strat_name, sample in samples.items():

        x = list(range(len(sample)))
        y = sample
        ax.scatter(
            x,
            y,
            s=15,
            marker=STRATEGY_MARKERS.get(strat_name, "o"),
            label=strat_name,
            color=STRATEGY_COLORS.get(strat_name, None),
        )

    ax.set_xlabel("Sample index")
    ax.set_ylabel(ylabel)
    if title is None:
        title = f"Sample plot (throughput)"
    ax.set_title(title)
    ax.grid(True, alpha=0.3)
    ax.legend()

    plt.tight_layout()
    if path:
        fig.savefig(path)
        plt.close(fig)
    else:
        plt.show()


def plot_sample(
        sample: list,
        title: Optional[str] = None,
        outlier_indices: Optional[Iterable[int]] = None,
        path: str = None
) -> None:

    x = list(range(len(sample)))

    y = sample
    ylabel = "Throughput (elements / s)"

    fig, ax = plt.subplots()
    ax.scatter(x, y, s=15, color="C0", label="samples")

    median_y = statistics.median(y)

    ax.axhline(median_y, color="orange", linestyle="--", linewidth=1.5, label="median")

    # Highlight outliers if provided
    if outlier_indices is not None:
        outlier_indices = list(outlier_indices)
        x_out = [x[i] for i in outlier_indices]
        y_out = [y[i] for i in outlier_indices]
        ax.scatter(x_out, y_out, s=25, color="red", label="outliers")

    ax.set_xlabel("Sample index")
    ax.set_ylabel(ylabel)
    if title is None:
        title = f"Sample plot (throughput)"
    ax.set_title(title)
    ax.grid(True, alpha=0.3)

    if outlier_indices is not None:
        ax.legend()

    plt.tight_layout()
    if path:
        fig.savefig(path)
        plt.close()
    else:
        plt.show()


def plot_sample_with_outliers(sample: list,
                              title: Optional[str] = None,
                              path: str = None
                              ):
    is_outlier, fences = tukey_outliers_from_sample(sample)
    outlier_indices = [i for i, flag in enumerate(is_outlier) if flag]
    plot_sample(sample, title, outlier_indices, path)
    print(f"Plotting and saving: {path}")
