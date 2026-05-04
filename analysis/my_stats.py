import statistics
from typing import List


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