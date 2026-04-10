use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::Event;
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

    fn window_closed<'a>(&mut self, window_index: &str, cutoff_or_open: &CutoffOrOpen, report: bool) {

        let snapshot = match cutoff_or_open {
            CutoffOrOpen::Cutoff(cutoff) => {
                // Clone all current item content
                let snapshot = self.content.clone();

                // Remove all content before the cutoff
                self.content.retain(|e| after_open(cutoff, &e.ts));
                snapshot
            }
            CutoffOrOpen::Open(open) => {
                // Clone all current item content
                self.content.iter().filter(|e| after_open(open, &e.ts)).cloned().collect()
            }
        };

        if report {
            self.consume_window(window_index, RcContainer(snapshot));
        }
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