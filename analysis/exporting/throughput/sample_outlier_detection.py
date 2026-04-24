from typing import Mapping, Sequence

import numpy as np


def tukey_outliers_from_sample(
        sample: Mapping,
        metric: str = "times_per_iter",  # or "throughputs"
        k: float = 1.5,
):
    """
    Compute Tukey outliers for a Criterion sample.

    Parameters:
        sample: dict with "times_per_iter" or "throughputs" list
        metric: which list to use ("times_per_iter" or "throughputs")
        k: fence factor (1.5 for mild, 3.0 for severe)

    Returns:
        is_outlier: list[bool] same length as data, True if sample is an outlier
        fences: (lower_fence, upper_fence)
    """
    data: Sequence[float] = sample[metric]
    arr = np.asarray(data, dtype=float)

    q1 = np.percentile(arr, 25)
    q3 = np.percentile(arr, 75)
    iqr = q3 - q1

    lower_fence = q1 - k * iqr
    upper_fence = q3 + k * iqr

    is_outlier = [(x < lower_fence) or (x > upper_fence) for x in arr]
    return is_outlier, (lower_fence, upper_fence)


