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