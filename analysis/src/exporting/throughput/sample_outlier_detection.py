from typing import Mapping, Sequence, List

import numpy as np


def tukey_outliers_from_sample(
        sample: list,
        k: float = 1.5,
):
    """
    Compute Tukey outliers for a Criterion sample.

    Parameters:
        sample: dict with "times_per_iter" or "throughputs" list
        k: fence factor (1.5 for mild, 3.0 for severe)

    Returns:
        is_outlier: list[bool] same length as data, True if sample is an outlier
        fences: (lower_fence, upper_fence)
    """
    arr = np.asarray(sample, dtype=float)

    q1 = np.percentile(arr, 25)
    q3 = np.percentile(arr, 75)
    iqr = q3 - q1

    lower_fence = q1 - k * iqr
    upper_fence = q3 + k * iqr

    is_outlier = [(x < lower_fence) or (x > upper_fence) for x in arr]
    return is_outlier, (lower_fence, upper_fence)


def remove_outliers_tukey(sample: List[float]) -> List[float]:
    """
    Return a new list with Tukey outliers removed.
    The original `sample` is not modified.
    """
    is_outlier, fences = tukey_outliers_from_sample(sample)
    # Keep only values where the corresponding flag is False
    return [x for x, flag in zip(sample, is_outlier) if not flag]
