STRATEGIES = ["slice", "rc", "arc", "clone", "legacy"]

ESTIMATES = {
    "thr_mean": "Mean throughput",
    "thr_median": "Median throughput"
}

STRATEGY_COLORS = {
    "slice": "tab:green",
    "rc": "tab:red",
    "arc": "tab:orange",
    "clone": "tab:blue",
    "legacy": "tab:purple",
}

STRATEGY_MARKERS = {
    "slice": "^",   # triangle up
    "rc": "s",       # square
    "arc": "D",      # diamond
    "clone": "o",    # circle
    "legacy": "X",   # filled x
}