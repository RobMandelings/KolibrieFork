use crate::WindowParams;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workload {
    pub name: String,
    pub nr_events: usize,
    pub windows: Vec<S2RWindowConfig>,
}

use serde_json;
use std::fs::File;
use std::io::Write;
use crate::prototype::event::Time;
use crate::prototype::helpers::wc_struct;
use crate::prototype::window_params::S2RWindowConfig;

pub fn write_workload_to_file(workload: &Workload, path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(workload)?; // or to_string for compact [web:379][web:381]
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn create_workload(nr_windows: usize, nr_events: usize, size: Time, slide: Time) -> Workload {
    let window_config = WindowParams {
        size,
        slide,
        offset: 0
    };

    let windows = {
        match nr_windows {
            1 => {
                vec![wc_struct(window_config)]
            },
            2 => {
                vec![wc_struct(window_config.clone()), wc_struct(window_config)]
            },
            _ => panic!("Not configured")
        }
    };

    Workload {
        name: format!("windows={nr_windows},size={size},slide={slide},events={nr_events}"),
        nr_events,
        windows,
    }
}

fn single_window_workloads() -> Vec<Workload> {
    let mut workloads = Vec::new();

    // event counts: 1_000, 10_000, 100_000
    for nr_windows in [1] {
        for &nr_events in &[1000, 10_000, 100_000] {
            for size in [1, 2, 4, 8, 16, 32, 64, 128] {
                for slide in [1, 5, 10, 20] {
                    workloads.push(create_workload(nr_windows, nr_events, size, slide));
                }
            }
        }
    }

    workloads
}

pub fn test_workloads() -> Vec<Workload> {

    let mut workloads = Vec::new();

    for size in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
        workloads.push(create_workload(1, 50_000, size, 1))
    }

    workloads
}

pub fn test_workload() -> Vec<Workload> {

    let mut workloads = Vec::new();

    for size in [1, 2, 4, 8, 16, 32, 64, 128] {
        workloads.push(create_workload(1, 50_000, size, 1))
    }

    workloads
}

pub fn default_workloads() -> Vec<Workload> {
    test_workload()
}