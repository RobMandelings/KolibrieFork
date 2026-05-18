from __future__ import annotations

import statistics
from typing import List, Dict
from scipy import stats


def thr_mean(sample: List[float]) -> float:
    return statistics.mean(sample)


def thr_median(sample: List[float]) -> float:
    return statistics.median(sample)


def thr_std_dev(sample: List[float]) -> float:
    if len(sample) > 1:
        return statistics.stdev(sample)
    return 0.0


def thr_std_err(sample: List[float]) -> float:
    n = len(sample)
    if n < 2:
        return 0.0
    return thr_std_dev(sample) / (n ** 0.5)


def thr_min(sample: List[float]) -> float:
    return min(sample)


def thr_max(sample: List[float]) -> float:
    return max(sample)


def thr_mean_ci_95(sample: List[float]) -> Dict[str, float | None]:
    """
    Two-sided 95% CI around the mean throughput.
    Returns dict with keys: lower, upper, margin.
    """
    n = len(sample)
    if n < 2:
        return {"lower": None, "upper": None, "margin": 0.0}

    mean_val = thr_mean(sample)
    se = thr_std_err(sample)

    t_crit = stats.t.ppf(0.975, df=n - 1)  # 95% two-sided
    margin = t_crit * se

    return {
        "lower": mean_val - margin,
        "upper": mean_val + margin,
        "margin": margin,
    }
