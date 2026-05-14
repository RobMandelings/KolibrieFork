use crate::prototype::event::Time;
use crate::{Event, WindowParams};
use indexmap::IndexMap;
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
    pub reserve: usize,
}

impl Workload {
    pub fn get_short_name(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{}",
            self.nr_windows,
            self.window.size,
            self.window.slide,
            self.nr_events,
            self.stream_config.spread,
            self.stream_config.offset,
            self.bytes,
            self.reserve
        )
    }
}

pub fn write_workload_to_file(workload: &Workload, path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(workload)?; // or to_string for compact [web:379][web:381]
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn create_events_for_workload<I, F>(workload: &Workload, initial_window_offset: Time, mut event_factory: F) -> Vec<Event<I>>
where
    F: FnMut(Time) -> Event<I>,
{
    (0..workload.nr_events as Time)
        .map(|i| event_factory(i * workload.stream_config.spread + workload.stream_config.offset + initial_window_offset))
        .collect()
}

pub fn create_workload(
    nr_windows: usize,
    nr_events: usize,
    spread: Time,
    event_ts_offset: Time,
    bytes: usize,
    size: Time,
    slide: Time,
    reserve: usize,
) -> Workload {
    let window_config = WindowParams {
        size,
        slide,
        offset: 0,
    };

    Workload {
        name: format!(
            "windows={nr_windows},size={size},slide={slide},events={nr_events},spread={spread},event_offset={event_ts_offset},bytes={bytes},reserve={reserve}"
        ),
        nr_events,
        stream_config: EventStreamConfig {
            spread,
            offset: event_ts_offset,
        },
        bytes,
        window: window_config,
        nr_windows,
        reserve,
    }
}

pub fn mk_workload(nr_events: usize, spread: Time, event_ts_offset: Time) -> Workload {
    Workload {
        name: "".to_string(),
        nr_events,
        stream_config: EventStreamConfig {
            spread,
            offset: event_ts_offset,
        },
        bytes: 0,
        window: WindowParams {
            size: 0,
            slide: 0,
            offset: 0,
        },
        nr_windows: 0,
        reserve: 0,
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
        let workload = mk_workload(5, 1, 0);
        let events = create_events_for_workload(&workload, 0, mk_event);
        let ts: Vec<Time> = events.iter().map(|e| e.ts).collect();
        assert_eq!(ts, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn timestamps_with_spread_five() {
        let workload = mk_workload(5, 5, 0);
        let events = create_events_for_workload(&workload, 0, mk_event);

        let ts: Vec<Time> = events.iter().map(|e| e.ts).collect();
        assert_eq!(ts, vec![0, 5, 10, 15, 20]);
    }

    #[test]
    fn timestamps_with_spread_five_offset_3() {
        let workload = mk_workload(5, 5, 3);
        let events = create_events_for_workload(&workload, 0, mk_event);

        let ts: Vec<Time> = events.iter().map(|e| e.ts).collect();
        assert_eq!(ts, vec![3, 8, 13, 18, 23]);
    }

    #[test]
    fn zero_events_produces_empty_vec() {
        let workload = mk_workload(0, 5, 0);
        let events = create_events_for_workload(&workload, 0, mk_event);

        assert!(events.is_empty());
    }

    #[test]
    fn timestamps_with_spread_two() {
        let workload = mk_workload(3, 2, 0);
        let events = create_events_for_workload(&workload, 0, mk_event);

        let ts: Vec<Time> = events.iter().map(|e| e.ts).collect();
        assert_eq!(ts, vec![0, 2, 4]);
    }

    #[test]
    fn timestamps_test_offset_1_ends_at_10() {
        let workload = mk_workload(10, 1, 1);
        let events = create_events_for_workload(&workload, 0, mk_event);

        let ts: Vec<Time> = events.iter().map(|e| e.ts).collect();
        assert_eq!(ts, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }
}
