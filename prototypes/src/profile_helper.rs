use dhat::Profiler;
use crate::bench_helpers::RunnerFactory;
use crate::workloads::Workload;

pub fn run_mem_profile(strategy_name: &str, setup_runner: &RunnerFactory)
{
    println!("Running memory profile for {strategy_name}");
    let runner= setup_runner();
    let _profiler = Profiler::new_heap();
    runner();
}