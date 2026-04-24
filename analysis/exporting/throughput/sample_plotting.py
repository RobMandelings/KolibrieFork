from __future__ import annotations

import statistics
from typing import Mapping, Literal, Optional, Iterable

from matplotlib import pyplot as plt

from constants import STRATEGY_COLORS
from exporting.throughput.sample_outlier_detection import tukey_outliers_from_sample


def extract_samples_by_strategy(workload: dict) -> dict:
    return {
        strategy: data["throughput"]["sample"]
        for strategy, data in workload["strategies"].items()
        if "throughput" in data and "sample" in data["throughput"]
    }


def plot_samples_grouped(
        workload: dict,
        mode: Literal["time", "throughput"] = "time",
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

    for strat_name, sample in samples.items():
        times = sample["times"]
        iters = sample["iters"]

        if len(times) != len(iters):
            raise ValueError(f"times and iters length mismatch for strategy {strat_name!r}")

        x = list(range(len(times)))

        if mode == "time":
            y = [t / i for t, i in zip(times, iters)]
            ylabel = "Time per iteration (ns)"
        elif mode == "throughput":
            if "throughputs" not in sample:
                raise ValueError(
                    f"sample['throughputs'] is missing for strategy {strat_name!r}; compute it first"
                )
            y = sample["throughputs"]
            ylabel = "Throughput (elements / s)"
        else:
            raise ValueError(f"Unknown mode: {mode!r}")

        ax.scatter(x, y, s=15, label=strat_name, color=STRATEGY_COLORS.get(strat_name, None))

    ax.set_xlabel("Sample index")
    ax.set_ylabel(ylabel)
    if title is None:
        title = f"Sample plot ({mode})"
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
        sample: Mapping,
        mode: Literal["time", "throughput"] = "time",
        title: Optional[str] = None,
        outlier_indices: Optional[Iterable[int]] = None,
        path: str = None
) -> None:
    """
    Plot a Criterion sample:
      - mode="time": y-axis = per-iteration time in nanoseconds
      - mode="throughput": y-axis = throughput (elements/sec), using sample["throughputs"]

    Assumes:
      sample["times"]      -> list of total sample times in ns
      sample["iters"]      -> list of iterations per sample
      sample["throughputs"] (optional) -> list of elements/sec per sample

    outlier_indices:
      Iterable of sample indices to highlight as outliers (plotted in red).
    """
    times = sample["times"]
    iters = sample["iters"]

    if len(times) != len(iters):
        raise ValueError("times and iters must have the same length")

    x = list(range(len(times)))

    if mode == "time":
        # per-iteration time in ns
        y = [t / i for t, i in zip(times, iters)]
        ylabel = "Time per iteration (ns)"
    elif mode == "throughput":
        if "throughputs" not in sample:
            raise ValueError("sample['throughputs'] is missing; compute it first")
        y = sample["throughputs"]
        ylabel = "Throughput (elements / s)"
    else:
        raise ValueError(f"Unknown mode: {mode!r}")

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
        title = f"Sample plot ({mode})"
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


def plot_sample_with_outliers(sample: Mapping,
                              mode: Literal["time", "throughput"] = "time",
                              title: Optional[str] = None,
                              path: str = None
                              ):
    is_outlier, fences = tukey_outliers_from_sample(sample)
    outlier_indices = [i for i, flag in enumerate(is_outlier) if flag]
    plot_sample(sample, mode, title, outlier_indices, path)
    print(f"Plotting and saving: {path}")
