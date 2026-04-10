use std::cell::RefCell;
use crate::{Event, IRI};
use crate::prototype::event::Time;
use crate::prototype::slide_strategy::{CutoffOrOpen, ItemsReport, WindowSnapshotStrategy};
use std::collections::{BTreeMap, HashMap};
use std::fmt::format;
use log::debug;

#[derive(Clone)]
pub struct ExpireContainer<'a, I> {
    pub last_timestamp_changed: Time,
    pub slice: &'a [I],
}

/// Continuous report: containing slice
pub type ContReport<'a, I> =
<SliceExpireStrategy<I> as WindowSnapshotStrategy<I>>::ReportType<'a>;

impl<'a, I> ExpireContainer<'a, I> {

    pub fn new(last_ts: Time, slice: &'a [I]) -> ExpireContainer<'a, I> {
        ExpireContainer {
            last_timestamp_changed: last_ts,
            slice
        }
    }
}

impl<I: 'static> ItemsReport<I> for ExpireContainer<'_, I> {
    fn get_last_timestamp_changed(&self) -> Time {
        self.last_timestamp_changed
    }

    fn iter_items(&self) -> impl Iterator<Item = &I> {
        self.slice.iter()
    }
}

/// Concrete slide_strategy: expire old events, report them as owned Events.
pub struct SliceExpireStrategy<I: Clone> {
    // Outer Vec: one entry per window
    // Inner Vec: consumers for that window
    consume_fns: HashMap<String, Vec<RefCell<Box<dyn FnMut(ExpireContainer<I>)>>>>,
    pub items: Vec<I>,
    pub timestamps: Vec<Time>,
}


impl<I: Clone> SliceExpireStrategy<I> {

    /// Take a slice of the event vector by finding the first position with a certain time stamp and last position
    pub fn get_index_for_ts(&self, open: Time) -> usize {
        self.timestamps.iter().position(|&ts| ts == open).expect(&format!("Should get index for timestamp {open}!"))
    }

    pub fn get_slice_for_ts(&self, open: Time) -> &[I] {
        let idx = self.get_index_for_ts(open);
        &self.items[..idx]
    }

    pub fn get_last_ts(&self) -> Time {
        *self.timestamps.get(self.timestamps.len() - 1).expect("Should get a timestamp")
    }
}

// TODO what if you allow references to be passed instead then? So it contains references?
// That would be weird
impl<I: Clone + 'static> WindowSnapshotStrategy<I> for SliceExpireStrategy<I> {

    type ReportType<'a> = ExpireContainer<'a, I>;

    fn new() -> Self {
        Self {
            consume_fns: HashMap::new(),
            items: Vec::new(),
            timestamps: Vec::new(),
        }
    }

    fn window_closed<'a>(
        &mut self,
        window_iri: &str,
        cutoff_or_open: &CutoffOrOpen,
        report: bool,
    ) {
        match cutoff_or_open {
            CutoffOrOpen::Cutoff(cutoff) => {
                if report {
                    let container = ExpireContainer::new(self.get_last_ts(), &self.items);
                    self.consume_window(window_iri, container);
                }

                let idx = self.get_index_for_ts(*cutoff);
                // Drain both the items and the timestamps vector
                let drained = self.items.drain(..idx).count();
                self.timestamps.drain(..idx);
                debug!("Drained {drained} elements, index was {idx}");

                debug!("All events before the cutoff timestamp {cutoff} are dropped. Remaining elements: {}", self.items.len());
            }
            CutoffOrOpen::Open(open) => {
                let slice = self.get_slice_for_ts(*open);
                if report {
                    let container = ExpireContainer::new(self.get_last_ts(), slice);
                    self.consume_window(window_iri, container);
                }
            }
        };
    }

    fn add_event(&mut self, event: Event<I>) {
        // Add to different vectors
        self.items.push(event.payload);
        self.timestamps.push(event.ts);
    }

    fn consume_fns(&self) -> &HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>> {
        &self.consume_fns
    }

    fn consume_fns_mut(&mut self) -> &mut HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>> {
        &mut self.consume_fns
    }

}

#[cfg(test)]
mod tests {
    use crate::prototype::slide_strategy::CutoffOrOpen::Open;
    use super::*;

    fn consume_fn() -> Box<dyn for<'a> Fn(ContReport<String>)> {
        Box::new(|_| {})
    }

    fn create_events() -> Vec<Event<String>> {
        let events = vec![
            Event::new(10, "A".to_string()),
            Event::new(20, "B".to_string()),
            Event::new(30, "C".to_string()),
        ];
        events
    }

    fn init_expire() -> SliceExpireStrategy<String> {
        let mut strat: SliceExpireStrategy<String> = SliceExpireStrategy::new();
        let events = create_events();
        for e in events {
            strat.add_event(e);
        }

        strat
    }

    fn init_expire_empty_consume() -> SliceExpireStrategy<String> {
        let mut strat = init_expire();
        strat.add_consumer("0", consume_fn());
        strat
    }

    #[test]
    fn expire_events_none_expired() {

        let strat = init_expire_empty_consume();
        let slice = strat.get_slice_for_ts(5); // cutoff before all
        assert_eq!(slice.len(), 3);
    }

    #[test]
    fn expire_events_some_expired() {
        let strat = init_expire_empty_consume();
        let slice = strat.get_slice_for_ts(25); // expire ts < 25 -> 10 and 20

        println!("{}", slice[0]);
        assert_eq!(slice.len(), 1);
        assert_eq!(slice[0], "C");
    }

    #[test]
    fn expire_events_all_expired() {
        let strat = init_expire_empty_consume();
        let slice = strat.get_slice_for_ts(100);
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn test_consumer() {
        let mut strat = init_expire();
        let consumer = Box::new(|report: ContReport<String>| {
            println!("{}", report.last_timestamp_changed);
            println!("{}", report.slice.join(","));
        });
        strat.add_consumer("0", consumer);
        strat.window_closed("0", &Open(20), true);
    }
}
