/*
 * Copyright © 2025 Volodymyr Khadzhaia
 * Copyright © 2025 Pieter Bonte
 * KU Leuven — Stream Intelligence Lab, Belgium
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * you can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[cfg(not(test))]
use log::{debug, warn}; // Use log crate when building application
use std::collections::hash_set::{IntoIter, Iter};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{f64, mem};
#[cfg(test)]
use std::{println as warn, println as debug};
use crate::rsp::s2r::reporting::Report;

pub mod reporting;

/// Tick is a dimension that explains what triggers the report evaluations.
/// Possible ticks are time-driven, tuple-driven, or batch-driven.
#[derive(Clone, Debug)]
pub enum Tick {
    TimeDriven,
    TupleDriven,
    BatchDriven,
}

impl Default for Tick {
    fn default() -> Self {
        Tick::TimeDriven
    }
}

/// Window is represented as an opening timestamp and closing timestamp
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct WindowBounds {
    open: usize,  // timestamp for when the window is opened
    close: usize, // timestamp for when the window is closed
}

impl WindowBounds {
    pub fn within_bounds(&self, event_time: usize) -> bool {
        self.open <= event_time && event_time <= self.close
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct WindowContent<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    elements: HashSet<I>,
    last_timestamp_changed: usize,
    origin: String,
}

impl<I> WindowContent<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    fn new() -> WindowContent<I> {
        WindowContent {
            elements: HashSet::new(),
            last_timestamp_changed: 0,
            origin: String::default(),
        }
    }
    fn new_with_origin(origin: &str) -> WindowContent<I> {
        WindowContent {
            elements: HashSet::new(),
            last_timestamp_changed: 0,
            origin: origin.to_string(),
        }
    }
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    fn add(&mut self, triple: I, ts: usize) {
        self.elements.insert(triple);
        self.last_timestamp_changed = ts;
    }
    pub fn get_last_timestamp_changed(&self) -> usize {
        self.last_timestamp_changed
    }

    pub fn iter(&self) -> Iter<'_, I> {
        self.elements.iter()
    }
    pub fn into_iter(mut self) -> IntoIter<I> {
        let map = mem::take(&mut self.elements);
        map.into_iter()
    }
}

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
            app_time: 0,
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
        self
            .active_windows
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

    fn add_item_to_active_windows(
        &mut self,
        event_item: &I,
        event_time: usize,
    ) -> () {
        self.active_windows = self.active_windows
            .clone()
            .into_iter()
            .filter_map(|(window, content)| {
                self.add_item_if_within_bounds(window, content, event_item, event_time)
            })
            .collect();
    }

    pub fn add_to_window(&mut self, event_item: I, event_time: usize) {
        self.set_active_windows_by_timestamp(&event_time);
        self.trigger_consume(event_time);

        // Why is this item added only after the scoping?
        self.add_item_to_active_windows(&event_item, event_time);
    }

    /// Update active_windows based on current event time
    /// So that only those windows are active that fit within the scope (current event time)
    fn set_active_windows_by_timestamp(&mut self, event_time: &usize) {
        // Both _temp are for debugging purposes it seems

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

/// Part of the Consumer struct
#[allow(dead_code)]
struct ConsumerInner<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    data: Mutex<Vec<WindowContent<I>>>,
}

#[allow(dead_code)]
struct Consumer<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    inner: Arc<ConsumerInner<I>>,
}

#[allow(dead_code)]
impl<I> Consumer<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    fn new() -> Consumer<I> {
        Consumer {
            inner: Arc::new(ConsumerInner {
                data: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Start listening for content sending in a different thread
    /// If content is received, push content to the consumer_temp (clone of inner consumer)
    fn start(&self, receiver: Receiver<WindowContent<I>>) {
        let consumer_temp = self.inner.clone();
        thread::spawn(move || loop {
            match receiver.recv() {
                // .revc() is a blocking operation (wait until you get result or err)
                Ok(content) => {
                    debug!("Found graph {:?}", content);
                    consumer_temp.data.lock().unwrap().push(content);
                }
                Err(_) => {
                    debug!("Shutting down!");
                    break;
                }
            }
        });
    }
    fn len(&self) -> usize {
        self.inner.data.lock().unwrap().len()
    }
}
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct WindowTriple {
    pub s: String,
    pub p: String,
    pub o: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use crate::rsp::s2r::reporting::ReportStrategy;

    #[test]
    fn test_window() {
        let mut report = Report::new();
        report.add(ReportStrategy::OnWindowClose);
        let mut window = CSPARQLWindow {
            width: 10,
            slide: 2,
            app_time: 0,
            t_0: 0,
            active_windows: HashMap::new(),
            report,
            tick: Tick::TimeDriven,
            consumer: None,
            callback: None,
            uri: "test_window".to_string(),
        };

        // When windows are reported, the receiver will receive window contents
        let receiver = window.register_consumer();

        // The consumer will consume the received window content
        let consumer = Consumer::new();
        consumer.start(receiver);

        for time in 0..10 {
            let triple = WindowTriple {
                s: format!("s{}", time),
                p: "p".to_string(),
                o: "o".to_string(),
            };

            window.add_to_window(triple, time);
        }

        window.stop();
        thread::sleep(Duration::from_secs(1));
        assert_eq!(5, consumer.len());
    }
    #[test]
    fn test_window_with_callback() {
        let mut report = Report::new();
        report.add(ReportStrategy::OnWindowClose);
        let mut window: CSPARQLWindow<WindowTriple> = CSPARQLWindow {
            width: 10,
            slide: 2,
            app_time: 0,
            t_0: 0,
            active_windows: HashMap::new(),
            report,
            tick: Tick::TimeDriven,
            consumer: None,
            callback: None,
            uri: "test_window".to_string(),
        };
        let recieved_data = Arc::new(Mutex::new(Vec::new()));
        let data_clone = Arc::clone(&recieved_data);
        let call_back = move |content| {
            println!("Content: {:?}", content);
            recieved_data.lock().unwrap().push(content);
        };
        window.register_callback(Box::new(call_back));

        for time in 0..10 {
            let triple = WindowTriple {
                s: format!("s{}", time),
                p: "p".to_string(),
                o: "o".to_string(),
            };
            window.add_to_window(triple, time);
        }

        window.stop();
        assert_eq!(5, data_clone.lock().unwrap().len());
    }
}
