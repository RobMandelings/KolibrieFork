use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::Event;
use crate::prototype::event::Time;
use crate::prototype::slide_strategy::{CutoffOrOpen, ItemsReport, WindowSnapshotStrategy};
use crate::prototype::window_bounds::after_open;

/// Concrete slide_strategy: expire old events, report them as owned Events.
pub struct RcStrategy<I> {
    // Outer Vec: one entry per window
    // Inner Vec: consumers for that window
    consume_fns: HashMap<String, Vec<RefCell<Box<dyn FnMut(RcContainer<I>)>>>>,
    pub(crate) content: Vec<Rc<Event<I>>>,
}

pub struct RcContainer<I>(pub Vec<Rc<Event<I>>>);

impl<I> Clone for RcContainer<I> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub type RcConsumer<I> = Box<dyn Fn(RcContainer<I>)>;

impl<I: 'static> ItemsReport<I> for RcContainer<I> {
    fn iter_items(&self) -> impl Iterator<Item=&I> {
        self.0
            .iter()
            .map(|rc| &rc.payload)
    }
}

impl<I: 'static> WindowSnapshotStrategy<I> for RcStrategy<I> {

    type ReportType<'a> = RcContainer<I>;

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

    fn report_window<'a>(&mut self, window_index: &str, open_time: Time) {
        let start = self.content.partition_point(|e| e.ts < open_time);
        let snapshot: Vec<_> = self.content[start..].to_owned();
        self.consume_window(window_index, RcContainer(snapshot));
    }

    fn drop_expired_events(&mut self, open_time: Time) {
        let mut cutoff = self.content.len();
        for i in 0..self.content.len() {

            // First time we encounter event after the open time, that is where we 'stop'
            if after_open(&open_time, &self.content[i].ts) {
                cutoff = i;
                break; // Break because this does not work
            }
        }

        self.content.drain(0..cutoff);
    }

    fn add_event(&mut self, event: Event<I>) {
        self.content.push(Rc::new(event))
    }

    fn consume_fns(&self) -> &HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>> {
        &self.consume_fns
    }

    fn consume_fns_mut(&mut self) -> &mut HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>> {
        &mut self.consume_fns
    }
}