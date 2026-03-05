pub trait SlidingWindow<I> {

    /// Updates the sliding window with specific application
    /// Events are added at the current application time
    /// This allows updating the sliding window and triggering accordingly
    fn update_app_time(&mut self, app_time: usize) -> ();

    /// Adds the event to the window at the current application time
    fn add_event(&mut self, event_item: I);

    fn add_to_window(&mut self, event_item: I, ts: usize) -> ();
}