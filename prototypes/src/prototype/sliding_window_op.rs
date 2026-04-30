#[cfg(test)]
mod tests;
mod perf_tests;
mod stream_content_tests;
mod multi_window_test;
mod test_nr_reports_generated;

use std::collections::HashMap;
use crate::prototype::event::Time;
use crate::prototype::slide_strategy::WindowSnapshotStrategy;
use crate::prototype::sliding_window_bounds::{compute_earliest_open_time, SlidingWindowBounds};
use crate::{Event, IRI};
use crate::prototype::slide_strategy::CutoffOrOpen::{Cutoff, Open};
use std::marker::PhantomData;
use std::rc::Rc;
use uuid::Uuid;
use crate::prototype::window_params::S2RWindowConfig;

pub struct SlidingWindowOperator<I, S>
where
    I: 'static,
    S: WindowSnapshotStrategy<I>,
{
    stream_iri: IRI,
    pub sliding_windows: HashMap<IRI, SlidingWindowBounds>,
    app_time: Time,
    strategy: S,
    _marker: PhantomData<I>,
}

impl<I, S> SlidingWindowOperator<I, S>
where
    I: 'static,
    S: WindowSnapshotStrategy<I>,
{

    pub fn single_window(config: S2RWindowConfig, strategy: S) -> Self {
        Self::new_default_iri(vec![config], strategy)
    }

    pub fn new_default_iri(configs: Vec<S2RWindowConfig>, strategy: S) -> Self {
        Self::new(format!("urn:stream:{}", Uuid::new_v4()), configs, strategy)
    }

    pub fn new(stream_iri: IRI, configs: Vec<S2RWindowConfig>, strategy: S) -> Self {
        let mut sliding_windows: HashMap<IRI, SlidingWindowBounds> = HashMap::new();

        for config in configs {
            let window_iri = config.window_iri.clone();
            let bounds = SlidingWindowBounds::new(config);

            if sliding_windows.insert(window_iri.clone(), bounds).is_some() {
                panic!("Duplicate window IRI detected: {}", window_iri);
            }
        }

        Self {
            stream_iri,
            sliding_windows,
            app_time: 0,
            strategy,
            _marker: PhantomData,
        }
    }

    /// Helper function that takes item separately for compatibility with the RSP Engine
    pub fn event_arrives_with_ts(&mut self, item: I, ts: Time) {
        self.event_arrives(Event::new(ts, item));
    }

    pub fn event_arrives(&mut self, event: Event<I>) {
        assert!(event.ts >= self.app_time); // Timestamps increase monotonically

        let mut tick_flag = false;
        if event.ts > self.app_time {
            // Time based tick
            tick_flag = true;
            self.app_time = event.ts;
        }

        let to_process: Vec<IRI> = self
            .sliding_windows
            .iter()
            .filter_map(|(iri, window)| {
                if window.slides_at(event.ts) {
                    Some(iri.clone())
                } else {
                    None
                }
            })
            .collect();

        for iri in to_process {
            let open_before = {
                let window = self
                    .sliding_windows
                    .get_mut(&iri)
                    .expect("IRI disappeared from sliding_windows");

                let open = window.active_bounds.open;
                window.slide(event.ts);
                open
            };

            let earliest_open_time = compute_earliest_open_time(self.sliding_windows.values());

            if earliest_open_time > open_before {
                self.strategy
                    .window_closed(&iri, &Cutoff(earliest_open_time), tick_flag);
            } else {
                self.strategy.window_closed(&iri, &Open(open_before), tick_flag);
            }
        }


        // for (iri, window) in &mut self.sliding_windows {
        //     if window.slides_at(event.ts) {
        //         let open = window.active_bounds.open;
        //         window.slide(event.ts);
        //         let earliest_open_time = compute_earliest_open_time(self.sliding_windows.values());
        //
        //         if earliest_open_time > open {
        //             // The earliest open time has changed. This means that the open time of the window that has closed
        //             // Was the earliest before. All events before the current earliest open time can be removed.
        //             self.strategy.window_closed(iri, &Cutoff(earliest_open_time), tick_flag);
        //         } else {
        //
        //             // No events should be deleted from memory. The earliest open time has not shifted
        //             self.strategy.window_closed(iri, &Open(open), tick_flag);
        //         }
        //     }
        // }

        self.strategy.add_event(event);
    }

    pub fn add_consumer(&mut self, window_iri: &str, consumer: Box<dyn for<'a> FnMut(S::ReportType<'a>)>) {
        if !self.sliding_windows.contains_key(window_iri) {
            panic!("Attempted to add consumer to non-existent window: {}", window_iri);
        }

        self.strategy.add_consumer(window_iri, consumer);
    }
}