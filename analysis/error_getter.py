from my_stats import thr_mean_ci_95


def get_throughput_std_dev(strat_data, workload_key, workload_data):
    throughput = strat_data.get("throughput")
    if throughput is None:
        raise Exception("Throughput is not present")

    estimates = throughput.get("estimates")
    if estimates is None:
        raise Exception("Estimates is not present")

    return estimates.get("thr_std_dev")


def get_throughput_std_err(strat_data, workload_key, workload_data):
    throughput = strat_data.get("throughput")
    if throughput is None:
        raise Exception("Throughput is not present")

    estimates = throughput.get("estimates")
    if estimates is None:
        raise Exception("Estimates is not present")

    return estimates.get("thr_std_err")


def get_throughput_conf_int_error(strat_data, workload_key, workload_data):
    throughput = strat_data.get("throughput")
    if throughput is None:
        raise Exception("Throughput is not present")

    sample = throughput.get("sample")
    if sample is None:
        raise Exception("Estimates is not present")

    return thr_mean_ci_95(sample)["margin"]
