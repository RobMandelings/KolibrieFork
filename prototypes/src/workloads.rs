use std::collections::HashMap;
use crate::{Event, WindowParams};
use crate::prototype::event::Time;
use crate::prototype::helpers::wc_struct;
use crate::prototype::window_params::S2RWindowConfig;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Write;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventStreamConfig {
    pub spread: Time,
    pub offset: Time,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workload {
    pub name: String,
    pub nr_events: usize,
    pub stream_config: EventStreamConfig,
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

pub fn create_events_for_workload<I, F>(
    workload: &Workload,
    mut event_factory: F,
) -> Vec<Event<I>>
where
    F: FnMut(Time) -> Event<I>,
{
    (0..workload.nr_events as Time)
        .map(|i| event_factory(i * workload.stream_config.spread + workload.stream_config.offset))
        .collect()
}

fn create_workload(
    nr_windows: usize,
    nr_events: usize,
    spread: Time,
    event_ts_offset: Time,
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
            "windows={nr_windows},size={size},slide={slide},events={nr_events},spread={spread},event_offset={event_ts_offset},bytes={bytes}"
        ),
        nr_events,
        stream_config: EventStreamConfig {
            spread,
            offset: event_ts_offset
        },
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
                    workloads.push(create_workload(nr_windows, nr_events, 1, 0, 0, size, slide));
                }
            }
        }
    }

    workloads
}

pub fn test_workloads() -> HashMap<String, Vec<Workload>> {
    let mut workloads1 = Vec::new();
        for size in [64] {
            for slide in [1, 2, 4, 8, 16, 32, 64] {
                // Event ts offset = size + 1 to directly trigger a report evaluation
                workloads1.push(create_workload(1, 50_000, 1,size + 1, 0, size, slide))
            }
        }

    // Always triggers another report evaluation
    let mut workloads2 = Vec::new();
    for size in [64] {
        for slide in [1, 2, 4, 8, 16, 32, 64] {
            // Event ts offset = size + 1 to directly trigger a report evaluation
            workloads2.push(create_workload(1, 50_000, slide,size + 1, 0, size, slide))
        }
    }

    let mut map: HashMap<String, Vec<Workload>> = HashMap::new();
    map.insert("spread_1".to_string(), workloads1);
    map.insert("spread_slide".to_string(), workloads2);
    map
}

pub fn vary_size() -> Vec<Workload> {
    let mut workloads = Vec::new();

    for bytes in [0, 32] {
        for size in [1, 2, 4, 8, 16, 32, 64, 128] {
            workloads.push(create_workload(1, 50_000, 1,0, bytes, size, 1))
        }
    }

    workloads
}

pub fn vary_slide() -> Vec<Workload> {
    let mut workloads = Vec::new();

    for bytes in [0] {
        for slide in [1, 2, 4, 8, 16, 32, 64] {
            workloads.push(create_workload(1, 50_000, 1,0, bytes, 64, slide))
        }
    }

    workloads
}

pub fn test_workload() -> Vec<Workload> {
    let mut workloads = Vec::new();

    for size in [1] {
        workloads.push(create_workload(5, 50_000, 1,0, 0, size, 1))
    }

    workloads
}

pub fn default_workloads() -> HashMap<String, Vec<Workload>> {
    test_workloads()
}

pub fn mk_workload(nr_events: usize, spread: Time, event_ts_offset: Time) -> Workload {
    Workload {
        name: "".to_string(),
        nr_events,
        stream_config: EventStreamConfig {
            spread,
            offset: event_ts_offset
        },
        bytes: 0,
        window: WindowParams {
            size: 0,
            slide: 0,
            offset: 0,
        },
        nr_windows: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A tiny factory that creates events with the given timestamp.
    fn mk_event(ts: Time) -> Event<()> {
        Event { ts, payload: () }
    }

    #[test]
    fn timestamps_with_spread_one() {
        let workload = mk_workload(5, 1,0 );
        let events = create_events_for_workload(&workload, mk_event);
        let ts: Vec<Time> = events.iter().map(|e| e.ts).collect();
        assert_eq!(ts, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn timestamps_with_spread_five() {
        let workload = mk_workload(5, 5, 0);
        let events = create_events_for_workload(&workload, mk_event);

        let ts: Vec<Time> = events.iter().map(|e| e.ts).collect();
        assert_eq!(ts, vec![0, 5, 10, 15, 20]);
    }

    #[test]
    fn timestamps_with_spread_five_offset_3() {
        let workload = mk_workload(5, 5, 3);
        let events = create_events_for_workload(&workload, mk_event);

        let ts: Vec<Time> = events.iter().map(|e| e.ts).collect();
        assert_eq!(ts, vec![3, 8, 13, 18, 23]);
    }

    #[test]
    fn zero_events_produces_empty_vec() {
        let workload = mk_workload(0, 5, 0);
        let events = create_events_for_workload(&workload, mk_event);

        assert!(events.is_empty());
    }

    #[test]
    fn timestamps_with_spread_two() {
        let workload = mk_workload(3, 2,0);
        let events = create_events_for_workload(&workload, mk_event);

        let ts: Vec<Time> = events.iter().map(|e| e.ts).collect();
        assert_eq!(ts, vec![0, 2, 4]);
    }
}