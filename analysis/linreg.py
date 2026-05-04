from typing import Dict, Any, List, Tuple
from scipy import stats


def linear_trend_per_strategy(
        series: Dict[str, Dict[str, Dict[str, Any]]],
        workload_order: List[str],
) -> Dict[str, Dict[str, float]]:
    """
    For each strategy in `series`, run a simple linear regression of
    y = value vs. x = position in workload_order.

    Parameters
    ----------
    series : dict
        Mapping: strategy -> { workload_key -> point_dict },
        where point_dict must contain at least "value".
    workload_order : list of str
        Ordered list of workload keys, same order as on the x-axis.

    Returns
    -------
    dict
        strategy -> { "slope": float, "intercept": float,
                      "p_value": float, "r_value": float }
    """
    results: Dict[str, Dict[str, float]] = {}

    for strategy, strat_points in series.items():
        x_vals: List[float] = []
        y_vals: List[float] = []

        for i, workload_key in enumerate(workload_order):
            point = strat_points.get(workload_key)
            if point is None:
                continue
            x_vals.append(float(i))  # x is position (0,1,2,...) or any numeric encoding you prefer
            y_vals.append(float(point["value"]))  # e.g. throughput

        if len(x_vals) < 2:
            # Not enough points to fit a line
            continue

        slope, intercept, r_value, p_value, std_err = stats.linregress(x_vals, y_vals)

        results[strategy] = {
            "slope": slope,
            "intercept": intercept,
            "p_value": p_value,
            "r_value": r_value,
        }

        print(
            f"Strategy={strategy}: slope={slope:.4g}, "
            f"p={p_value:.3g}, r={r_value:.4g}"
        )

    return results
