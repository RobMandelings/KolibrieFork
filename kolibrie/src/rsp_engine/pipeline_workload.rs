use std::fs::File;
use std::io::Write;
use serde::{Deserialize, Serialize};
use prototypes::prototype::event::Time;
use prototypes::prototype::helpers::wc_struct;
use prototypes::WindowParams;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineWorkload {
    pub name: String,
    pub nr_events: usize,
    pub window_params: WindowParams,
}

pub fn write_workload_to_file(workload: &PipelineWorkload, path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(workload)?; // or to_string for compact [web:379][web:381]
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn create_workload(nr_events: usize, size: Time, slide: Time) -> PipelineWorkload {
    let window_config = WindowParams {
        size,
        slide,
        offset: 0
    };

    PipelineWorkload {
        name: format!("size={size},slide={slide},events={nr_events}"),
        nr_events,
        window_params: window_config
    }
}

fn single_window_workloads() -> Vec<PipelineWorkload> {
    let mut workloads = Vec::new();

    // event counts: 1_000, 10_000, 100_000
    for &nr_events in &[1000, 10_000, 100_000] {
        for size in [1, 2, 4, 8, 16, 32, 64, 128] {
            for slide in [1, 5, 10, 20] {
                workloads.push(create_workload(nr_events, size, slide));
            }
        }
    }

    workloads
}

pub fn test_workloads() -> Vec<PipelineWorkload> {

    let mut workloads = Vec::new();

    for size in [1, 2, 4, 8, 16, 32, 64] {
        workloads.push(create_workload(1000, size, 1))
    }

    workloads
}

pub fn test_workload() -> Vec<PipelineWorkload> {

    let mut workloads = Vec::new();

    for size in [1] {
        workloads.push(create_workload(1000, size, 1))
    }

    workloads
}

pub fn default_workloads() -> Vec<PipelineWorkload> {
    test_workloads()
}