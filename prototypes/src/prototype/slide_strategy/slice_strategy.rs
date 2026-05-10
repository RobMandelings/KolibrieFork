use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::debug_helper::push_tracking;
use crate::Event;
use crate::prototype::event::Time;
use crate::prototype::slide_strategy::{CutoffOrOpen, ItemsReport, WindowSnapshotStrategy};
use crate::prototype::window_bounds::after_open;

/// Concrete slide_strategy: expire old events, report them as owned Events.
pub struct SliceStrategy<I: Clone> {
    // Outer Vec: one entry per window
    // Inner Vec: consumers for that window
    consume_fns: HashMap<String, Vec<RefCell<Box<dyn FnMut(SliceContainer<I>)>>>>,
    pub content: Vec<Event<I>>,
}

impl<I: Clone> SliceStrategy<I> {

    /// Take a slice of the event vector by finding the first position with a certain time stamp and last position
    pub fn slice_by_ts(&self, open: Time) -> &[Event<I>] {
        let lo = self.content.partition_point(|e| !after_open(&open, &e.ts));
        &self.content[lo..]
    }
}

pub type SliceConsumer<I> = Box<dyn Fn(SliceContainer<I>)>;
pub struct SliceContainer<'a, I>(pub &'a [Event<I>]);

impl<I> Clone for SliceContainer<'_, I> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<I: 'static> ItemsReport<I> for SliceContainer<'_, I> {
    fn get_last_timestamp_changed(&self) -> Time {
        self.0.last().unwrap().ts
    }

    fn iter_items(&self) -> impl Iterator<Item=&I> {
        self.0
            .iter()
            .map(|rc| &rc.payload)
    }
}

impl<I: Clone + 'static> WindowSnapshotStrategy<I> for SliceStrategy<I> {
    type ReportType<'a> = SliceContainer<'a, I>;

    fn new() -> Self {
        Self {
            consume_fns: HashMap::new(),
            content: Vec::new(),
        }
    }

    fn with_capacity(reserve: usize) -> Self {
        Self {
            consume_fns: HashMap::new(),
            content: Vec::with_capacity(reserve)
        }
    }

    fn report_window<'a>(&mut self, window_iri: &str, open_time: Time) {
        let slice = self.slice_by_ts(open_time);
        self.consume_window(window_iri, SliceContainer(slice));
    }

    fn drop_expired_events(&mut self, open_time: Time) {
        // println!("Before expired: {}", self.content.capacity());
        self.content.retain(|e| after_open(&open_time, &e.ts));
        // println!("After expired: {}", self.content.capacity());
    }

    fn add_event(&mut self, event: Event<I>) {
        push_tracking(&mut self.content, event, "Slice");
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
    use crate::make_string_event;
    use super::*;

    fn consume_fn() -> Box<dyn for<'a> Fn(SliceContainer<String>)> {
        Box::new(|_| {})
    }

    #[test]
    fn expire_events_none_expired() {
        let mut wc: SliceStrategy<String> = SliceStrategy::new();

        wc.content = vec![make_string_event(10), make_string_event(20), make_string_event(30)];
        wc.add_consumer("0", consume_fn());
        let slice = wc.slice_by_ts(5); // cutoff before all

        assert_eq!(slice.len(), 3);
    }

    #[test]
    fn expire_events_some_expired() {
        let mut wc = SliceStrategy::new();
        wc.content = vec![make_string_event(10), make_string_event(20), make_string_event(30)];
        wc.add_consumer("0", consume_fn());
        let slice = wc.slice_by_ts(25); // expire ts < 25 -> 10 and 20

        assert_eq!(slice.len(), 1);
        assert_eq!(slice[0].ts, 30);
    }

    #[test]
    fn expire_events_all_expired() {
        let mut wc = SliceStrategy::new();
        wc.content = vec![make_string_event(10), make_string_event(20), make_string_event(30)];
        wc.add_consumer("0", consume_fn());
        let slice = wc.slice_by_ts(100);
        assert_eq!(slice.len(), 0);
    }
}