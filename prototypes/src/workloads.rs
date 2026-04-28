use crate::WindowParams;
use crate::prototype::event::Time;
use crate::prototype::helpers::wc_struct;
use crate::prototype::window_params::S2RWindowConfig;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Write;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workload {
    pub name: String,
    pub nr_events: usize,
    pub bytes: usize,
    pub window: WindowParams,
    pub nr_windows: usize,
}

pub fn write_workload_to_file(workload: &Workload, path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(workload)?; // or to_string for compact [web:379][web:381]
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn create_workload(
    nr_windows: usize,
    nr_events: usize,
    bytes: usize,
    size: Time,
    slide: Time,
) -> Workload {
    let window_config = WindowParams {
        size,
        slide,
        offset: 0,
    };

    Workload {
        name: format!(
            "windows={nr_windows},size={size},slide={slide},events={nr_events},bytes={}",
            bytes
        ),
        nr_events,
        bytes,
        window: window_config,
        nr_windows
    }
}

fn single_window_workloads() -> Vec<Workload> {
    let mut workloads = Vec::new();

    // event counts: 1_000, 10_000, 100_000
    for nr_windows in [1] {
        for &nr_events in &[1000, 10_000, 100_000] {
            for size in [1, 2, 4, 8, 16, 32, 64, 128] {
                for slide in [1, 5, 10, 20] {
                    workloads.push(create_workload(nr_windows, nr_events, 0, size, slide));
                }
            }
        }
    }

    workloads
}

pub fn test_workloads() -> Vec<Workload> {
    let mut workloads = Vec::new();

    for bytes in [0, 32, 64] {
        for size in [1, 2, 4, 8, 16, 32, 64] {
            for slide in [1, 2, 4, 8, 16, 32, 64] {
                workloads.push(create_workload(1, 50_000, bytes, size, slide))
            }
        }
    }

    workloads
}

pub fn vary_size() -> Vec<Workload> {
    let mut workloads = Vec::new();

    for bytes in [0, 32] {
        for size in [1, 2, 4, 8, 16, 32, 64, 128] {
            workloads.push(create_workload(1, 50_000, bytes, size, 1))
        }
    }

    workloads
}

pub fn vary_slide() -> Vec<Workload> {
    let mut workloads = Vec::new();

    for bytes in [0] {
        for slide in [1, 2, 4, 8, 16, 32, 64] {
            workloads.push(create_workload(1, 50_000, bytes, 64, slide))
        }
    }

    workloads
}

pub fn test_workload() -> Vec<Workload> {
    let mut workloads = Vec::new();

    for size in [1] {
        workloads.push(create_workload(5, 50_000, 0, size, 1))
    }

    workloads
}

pub fn default_workloads() -> Vec<Workload> {
    test_workloads()
}
