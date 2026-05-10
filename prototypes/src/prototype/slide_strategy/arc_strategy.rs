use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use crate::Event;
use crate::prototype::event::Time;
use crate::prototype::slide_strategy::{CutoffOrOpen, ItemsReport, WindowSnapshotStrategy};
use crate::prototype::window_bounds::after_open;

/// Concrete slide_strategy: expire old events, report them as owned Events.
pub struct ArcStrategy<I> {
    // Outer Vec: one entry per window
    // Inner Vec: consumers for that window
    consume_fns: HashMap<String, Vec<RefCell<Box<dyn FnMut(ArcContainer<I>)>>>>,
    pub(crate) content: Vec<Arc<Event<I>>>,
}

pub struct ArcContainer<I>(pub Vec<Arc<Event<I>>>);

impl<I> Clone for ArcContainer<I> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<I: 'static> ItemsReport<I> for ArcContainer<I> {
    fn iter_items(&self) -> impl Iterator<Item=&I> {
        self.0
            .iter()
            .map(|rc| &rc.payload)
    }
}

impl<I: 'static> WindowSnapshotStrategy<I> for ArcStrategy<I> {
    type ReportType<'a> = ArcContainer<I>;

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
        let snapshot = self.content.iter().filter(|e| after_open(&open_time, &e.ts)).cloned().collect();
        self.consume_window(window_iri, ArcContainer(snapshot));
    }

    fn drop_expired_events(&mut self, open_time: Time) {
        self.content.retain(|e| after_open(&open_time, &e.ts));
    }

    fn add_event(&mut self, event: Event<I>) {
        self.content.push(Arc::new(event))
    }

    fn consume_fns(&self) -> &HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>> {
        &self.consume_fns
    }

    fn consume_fns_mut(&mut self) -> &mut HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>> {
        &mut self.consume_fns
    }
}