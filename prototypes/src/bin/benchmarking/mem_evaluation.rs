use std::{env, fs};
use std::path::{Path, PathBuf};
use prototypes::{run_mem_profile, run_strategy_arc, run_strategy_expire, run_strategy_refcount, WindowParams, WINDOW_CONFIGS};
use dhat::Alloc;

#[global_allocator]
static ALLOC: Alloc = Alloc;

// Might later extend beyond WindowParams, e.g. to also specify an event generator for example
fn get_configuration(config_index: usize) -> &'static WindowParams {
    &WINDOW_CONFIGS[config_index]
}

pub(crate) fn move_profile_file(strat: &str, group_path: &Path) {
    let dir = group_path.join("memory");
    // create mem_profiles/workload_name if needed
    fs::create_dir_all(&dir).expect("failed to create mem_profiles dir");

    let dest = dir.join(format!("{strat}.json"));

    fs::rename("dhat-heap.json", &dest).expect("failed to move dhat-heap.json");
}