use dhat::Profiler;
use crate::workloads::Workload;

pub fn run_mem_profile<F>(strategy_name: &str, runner: F, workload: &Workload)
where
    F: FnOnce(&Workload),
{
    println!("Running memory profile for {strategy_name}");

    let _profiler = Profiler::new_heap();

    runner(workload);
}