use crate::rsp::s2r::reporting::Report;
use crate::rsp::s2r::Tick;
use std::fmt::Debug;
use std::hash::Hash;

pub trait SlidingWindow<I> {
    /// Updates the sliding window with specific application
    /// Events are added at the current application time
    /// This allows updating the sliding window and triggering accordingly
    fn update_app_time(&mut self, app_time: usize) -> ();

    /// Adds the event to the window at the current application time
    fn add_event(&mut self, event_item: I);

    fn add_to_window(&mut self, event_item: I, ts: usize) -> ();

    /// Checks whether the next tick is triggered based on the configured Tick
    fn check_next_tick(&self, ts: usize) -> bool;


}

pub struct SlidingWindowCore<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    width: usize,
    slide: usize,
    t_0: usize,
    report: Report<I>,
    tick: Tick,
    app_time: usize, // Current application time: the time of the latest event item that was processed.
}
