use crate::rsp::s2r::reporting::Report;
use crate::rsp::s2r::Tick;
use log::{debug, warn};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::rsp::s2r::sliding_window::SlidingWindow;
use crate::rsp::s2r::window::{WindowBounds, WindowContent};

pub struct CSPARQLWindow<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    width: usize,
    slide: usize,
    t_0: usize,
    active_windows: HashMap<WindowBounds, WindowContent<I>>, // Each 'window' is a unique key
    report: Report<I>,
    tick: Tick,
    app_time: usize, // Current application time: the time of the latest event item that was processed.
    consumer: Option<Sender<WindowContent<I>>>,
    // Make callbacks Send so they can be safely transferred to worker threads
    callback: Option<Box<dyn FnMut(WindowContent<I>) -> () + Send + 'static>>,
    uri: String,
}

impl<I> SlidingWindow<I> for CSPARQLWindow<I> where
    I: Eq + PartialEq + Clone + Debug + Hash + Send {

    fn update_app_time(&mut self, app_time: usize) -> () {
        self.create_missing_windows_for_timestamp(&app_time);
        self.trigger_consume(app_time);
    }

    fn add_event(&mut self, event_item: I) {
        todo!()
    }

    fn add_to_window(&mut self, event_item: I, ts: usize) {
        self.update_app_time(ts);
        // Why is this item added only after the scoping?
        self.add_item_to_active_windows(&event_item, ts);
    }
}

/// Represents a sliding window where a consumer gets send the window contents based on reporting strategies
impl<I> CSPARQLWindow<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    pub fn new(
        width: usize,
        slide: usize,
        report: Report<I>,
        tick: Tick,
        uri: String,
    ) -> CSPARQLWindow<I> {
        CSPARQLWindow {
            slide,
            width,
            t_0: 0,
            app_time: 0, // app_time stands for "application time" - it tracks the latest timestamp that the application has processed so far
            report,
            consumer: None,
            active_windows: HashMap::new(),
            tick,
            callback: None,
            uri,
        }
    }

    /// `Returns`:
    ///     - Some(Content): If the item fits within window bounds, the item is added to the ContentContainer and the Content is returned.
    ///     - None: If the item does not fit within window bounds, nothing is added to the ContentContainer
    fn add_item_if_within_bounds(
        &self,
        window: WindowBounds,
        mut content: WindowContent<I>,
        event_item: &I,
        event_time: usize,
    ) -> Option<(WindowBounds, WindowContent<I>)> {
        debug!(
            "Processing Window [{:?}, {:?}) for element ({:?},{:?})",
            window.open, window.close, event_item, event_time
        );

        if window.within_bounds(event_time) {
            debug!(
                "Adding element [{:?}] to Window [{:?},{:?})",
                event_item, window.open, window.close
            );

            // Add the item if it fits within bounds
            content.add(event_item.clone(), event_time);
            Some((window, content))
        } else {
            debug!(
                "Scheduling for Eviction [{:?},{:?})",
                window.open, window.close
            );
            None
        }
    }

    fn get_latest_window_to_report(&mut self, event_time: usize) -> Option<WindowBounds> {
        // Gets the latest window that is ready to be reported
        self.active_windows
            .iter()
            .filter(|(window, content)| {
                self.report
                    .should_report_window(window, content, event_time)
            })
            .max_by(|(w1, _), (w2, _)| w1.close.cmp(&w2.close))
            .map(|(window, _)| window.clone()) // key is Window, so clone it
    }

    fn trigger_consume(&mut self, event_time: usize) {
        let max = self.get_latest_window_to_report(event_time);
        if let Some(max_window) = max {
            match self.tick {
                Tick::TimeDriven => {
                    if event_time > self.app_time {
                        self.app_time = event_time;
                        // notify consumers
                        debug!("Window triggers! {:?}", max_window);

                        // The content that will be send to the consumer
                        let content_for_window = self.active_windows.get(&max_window).unwrap();

                        // multithreaded consumer using channel
                        if let Some(sender) = &self.consumer {
                            if let Err(e) = sender.send(content_for_window.clone()) {
                                warn!("Failed to send window content to consumer: {:?}", e);
                            }
                        }
                        // single threaded consumer using callback
                        if let Some(call_back) = &mut self.callback {
                            (call_back)(content_for_window.clone());
                        }
                    }
                }
                _ => (), // Not implemented yet?
            };
        }
    }

    fn add_item_to_active_windows(&mut self, event_item: &I, event_time: usize) -> () {
        self.active_windows = self
            .active_windows
            .clone()
            .into_iter()
            .filter_map(|(window, content)| {
                self.add_item_if_within_bounds(window, content, event_item, event_time)
            })
            .collect();
    }

    /// Update active_windows based on current event time
    /// So that only those windows are active that fit within the scope (current event time)
    fn create_missing_windows_for_timestamp(&mut self, event_time: &usize) {

        // Smallest right bound of the window that is still >= your current event_time
        let c_sup = ((*event_time as f64 - self.t_0 as f64).abs() / (self.slide as f64)).ceil()
            * self.slide as f64;

        // The open-timestamp (left bound) of the window for the leftmost window that still fits in event
        // long o_i = c_sup - width;
        let mut cur_left_bound = c_sup - self.width as f64;
        debug!(
            "Calculating the Windows to Open. First one opens at [{:?}] and closes at [{:?}]",
            cur_left_bound, c_sup
        );
        // log.debug("Calculating the Windows to Open. First one opens at [" + o_i + "] and closes at [" + c_sup + "]");
        //
        loop {
            debug!(
                "Computing Window [{:?},{:?}) if absent",
                cur_left_bound,
                (cur_left_bound + self.width as f64)
            );

            // Define a new window based on open and close parameters
            let window = WindowBounds {
                open: cur_left_bound as usize,
                close: (cur_left_bound + self.width as f64) as usize,
            };

            // If such a window does not yet exist, insert it to the list of active windows
            if let None = self.active_windows.get(&window) {
                self.active_windows
                    .insert(window, WindowContent::new_with_origin(&self.uri));
            }

            // Slide the window so that in the next iteration, you can create a new Window object that can be added to the Vec<Window> struct.
            cur_left_bound += self.slide as f64;
            if cur_left_bound > *event_time as f64 {
                break;
            }
        }
    }

    /// Creates a new channel with send and receiver, updates the consumer to allow for sending
    /// Returns the receiver, so that when self.consumer.send() is used, the receiver is able to receive.
    pub fn register_consumer(&mut self) -> Receiver<WindowContent<I>> {
        // Create new channel that carries ContentContainer values
        let (send, recv) = channel::<WindowContent<I>>();
        self.consumer.replace(send);
        recv
    }

    /// Registers which callback function to call when you use self.callback
    pub fn register_callback(
        &mut self,
        function: Box<dyn FnMut(WindowContent<I>) -> () + Send + 'static>,
    ) {
        self.callback.replace(function);
    }

    /// Push window content clones to the callback and consumer
    pub fn flush(&mut self) {
        for (_, content) in &self.active_windows {
            if let Some(call_back) = &mut self.callback {
                (call_back)(content.clone());
            }
            if let Some(sender) = &self.consumer {
                let _ = sender.send(content.clone());
            }
        }
    }

    /// Sets consumer to none (nothing to consume, so its inactive)
    pub fn stop(&mut self) {
        self.consumer.take();
    }
}
