use std::cell::RefCell;
use std::collections::HashMap;
use crate::prototype::event::Time;
use crate::Event;

pub mod arc_strategy;
pub mod clone_strategy;
pub mod slice_strategy;
pub mod slice_expire_strategy;
pub mod iter_expire_strategy;
pub mod rc_strategy;
pub mod strategies_report_only;
pub mod event_arrives;

pub enum CutoffOrOpen {
    Cutoff(Time),
    Open(Time),
}

pub trait ItemsReport<I: 'static> {
    fn get_last_timestamp_changed(&self) -> Time {
     0
    }

    fn iter_items<>(&self) -> impl Iterator<Item=&I>;
}

/// Strategy for sliding a window and producing some report.
pub trait WindowSnapshotStrategy<I>
where
    I: 'static,
{
    type ReportType<'a>: Clone + ItemsReport<I>;

    fn new() -> Self;

    fn with_capacity(reserve: usize) -> Self;

    /// Slides the given sliding window to match the timestamp
    /// Report: whether to report the window content at closing time
    /// (in the semantics 'right before the slide happens')
    fn report_window<'a>(
        &mut self,
        window_iri: &str,
        open_time: Time,
    );

    fn drop_expired_events(&mut self, threshold: Time);

    fn add_event(&mut self, event: Event<I>);

    fn consume_fns(&self) -> &HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>>;

    fn consume_fns_mut(&mut self) -> &mut HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>>;

    /// Add a consumer for a specific window.
    fn add_consumer(&mut self, window_iri: &str, consumer: Box<dyn for<'a> FnMut(Self::ReportType<'a>)>,) {
        self.consume_fns_mut()
            .entry(window_iri.to_string())
            .or_default()
            .push(RefCell::new(consumer));
    }

    fn consume(&self, window_iri: &str, consumer_index: usize, snapshot: Self::ReportType<'_>) {
        if let Some(consumers) = self.consume_fns().get(window_iri) {
            consumers[consumer_index].borrow_mut()(snapshot);
        }
    }

    fn consume_window(&self, window_iri: &str, report: Self::ReportType<'_>) {

        // debug!("Sending report! Number of elems: {}", report.content.len());
        let len = match self.consume_fns().get(window_iri) {
            Some(c) if !c.is_empty() => c.len(),
            _ => return,
        };

        for i in 0..len - 1 {
            self.consume(window_iri, i, report.clone());
        }

        self.consume(window_iri, len - 1, report);
    }
}
