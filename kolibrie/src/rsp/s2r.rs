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

/// Reporting Strategies define the conditions under which the engine emits the content of the window.
#[derive(Clone, Debug)]
pub enum ReportStrategy {
    NonEmptyContent,
    OnContentChange,
    OnWindowClose,
    Periodic(usize),
}
impl Default for ReportStrategy {
    fn default() -> Self {
        ReportStrategy::OnWindowClose
    }
}

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

pub struct Report<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    strategies: Vec<ReportStrategy>,
    last_change: ContentContainer<I>,
}

impl<I> Report<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    pub fn new() -> Report<I> {
        Report {
            strategies: Vec::new(), // Reporting strategies to consider when checking whether window should be reported
            last_change: ContentContainer::new(), // Used for the OnContentChange reporting strategy
        }
    }

    /// Adds a new reporting strategy to the report.
    pub fn add(&mut self, strategy: ReportStrategy) {
        self.strategies.push(strategy);
    }

    /// Returns true if the window should be reported.
    /// This only happens when all reporting strategies within the Vec<ReportStrategy>
    /// say that reporting strategy should be 'true'
    pub fn report(&mut self, window: &Window, content: &ContentContainer<I>, ts: usize) -> bool {
        self.strategies.iter().all(|strategy| match strategy {
            ReportStrategy::NonEmptyContent => content.len() > 0,
            ReportStrategy::OnContentChange => {
                let comp = content.eq(&self.last_change);
                self.last_change = content.clone();
                comp
            }
            ReportStrategy::OnWindowClose => window.close < ts,
            ReportStrategy::Periodic(period) => ts % period == 0,
        })
    }
}

/// Window is represented as an opening timestamp and closing timestamp
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Window {
    open: usize, // timestamp for when the window is opened
    close: usize, // timestamp for when the window is closed
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ContentContainer<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    elements: HashSet<I>,
    last_timestamp_changed: usize,
    origin: String
}

impl<I> ContentContainer<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    fn new() -> ContentContainer<I> {
        ContentContainer {
            elements: HashSet::new(),
            last_timestamp_changed: 0,
            origin: String::default()
        }
    }
    fn new_with_origin(origin : &str) -> ContentContainer<I> {
        ContentContainer {
            elements: HashSet::new(),
            last_timestamp_changed: 0,
            origin: origin.to_string()
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
    active_windows: HashMap<Window, ContentContainer<I>>, // Each 'window' is a unique key
    report: Report<I>,
    tick: Tick,
    app_time: usize,
    consumer: Option<Sender<ContentContainer<I>>>,
    // Make callbacks Send so they can be safely transferred to worker threads
    call_back: Option<Box<dyn FnMut(ContentContainer<I>) -> () + Send + 'static>>,
    uri: String
}

/// Represents a sliding window where a consumer gets send the window contents based on reporting strategies
impl<I> CSPARQLWindow<I>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    pub fn new(width: usize, slide: usize, report: Report<I>, tick: Tick, uri: String) -> CSPARQLWindow<I> {
        CSPARQLWindow {
            slide,
            width,
            t_0: 0,
            app_time: 0,
            report,
            consumer: None,
            active_windows: HashMap::new(),
            tick,
            call_back: None,
            uri
        }
    }

    pub fn add_to_window(&mut self, event_item: I, ts: usize) {
        let event_time = ts;
        self.scope(&event_time);

        // Update all windows by adding the event_item to the corresponding windows
        // (Evict the windows for which the ts falls out of bounds)
        let updated_windows = self
            .active_windows
            .clone()
            .into_iter()
            // Update each window. Window is filtered from the list if it gets evicted (returns None).
            .filter_map(|(window, mut content)| {
                debug!(
                    "Processing Window [{:?}, {:?}) for element ({:?},{:?})",
                    window.open, window.close, event_item, ts
                );

                // Check whether event time is between two window times and add it if that is the case
                if window.open <= event_time && event_time <= window.close {
                    debug!(
                        "Adding element [{:?}] to Window [{:?},{:?})",
                        event_item, window.open, window.close
                    );
                    content.add(event_item.clone(), ts);
                    Some((window, content))
                } else {
                    debug!(
                        "Scheduling for Eviction [{:?},{:?})",
                        window.open, window.close
                    );
                    None
                }
            })
            .collect::<HashMap<Window, ContentContainer<I>>>();

        // Gets the latest window that should fire
        let max = self
            .active_windows
            .iter()
            .filter(|(window, content)| self.report.report(window, content, ts))
            .max_by(|(w1, _), (w2, _)| w1.close.cmp(&w2.close));

        if let Some(max_window) = max {
            match self.tick {
                Tick::TimeDriven => {
                    if ts > self.app_time {
                        self.app_time = ts;
                        // notify consumers
                        debug!("Window triggers! {:?}", max_window);
                        // multithreaded consumer using channel
                        if let Some(sender) = &self.consumer {
                            if let Err(e) = sender.send(max_window.1.clone()) {
                                warn!("Failed to send window content to consumer: {:?}", e);
                            }
                        }
                        // single threaded consumer using callback
                        if let Some(call_back) = &mut self.call_back {
                            (call_back)(max_window.1.clone());
                        }
                    }
                }
                _ => (), // Not implemented yet?
            };
        }

        self.active_windows = updated_windows;
    }


    /// Update active_windows based on current event time
    /// So that only those windows are active that fit within the scope (current event time)
    fn scope(&mut self, event_time: &usize) {

        // Both _temp are for debugging purposes it seems
        // long c_sup = (long) Math.ceil(((double) Math.abs(t_e - t0) / (double) slide)) * slide;
        let _temp = (*event_time as f64 - self.t_0 as f64).abs();
        let _temp = ((*event_time as f64 - self.t_0 as f64).abs() / (self.slide as f64)).ceil();

        // Smallest right bound of the window that is still >= your current event_time
        let c_sup = ((*event_time as f64 - self.t_0 as f64).abs() / (self.slide as f64)).ceil()
            * self.slide as f64;

        // The open-timestamp (left bound) of the window for the leftmost window that still fits in event
        // long o_i = c_sup - width;
        let mut o_i = c_sup - self.width as f64;
        debug!(
            "Calculating the Windows to Open. First one opens at [{:?}] and closes at [{:?}]",
            o_i, c_sup
        );
        // log.debug("Calculating the Windows to Open. First one opens at [" + o_i + "] and closes at [" + c_sup + "]");
        //
        loop {
            debug!(
                "Computing Window [{:?},{:?}) if absent",
                o_i,
                (o_i + self.width as f64)
            );

            // Define a new window based on open and close parameters
            let window = Window {
                open: o_i as usize,
                close: (o_i + self.width as f64) as usize,
            };

            // If such a window does not yet exist, insert it to the list of active windows
            if let None = self.active_windows.get(&window) {
                self.active_windows.insert(window, ContentContainer::new_with_origin(&self.uri));
            }

            // Slide the window so that in the next iteration, you can create a new Window object that can be added to the Vec<Window> struct.
            o_i += self.slide as f64;
            if o_i > *event_time as f64 {
                break;
            }
        }
    }

    /// Creates a new channel with send and receiver, updates the consumer to allow for sending
    /// Returns the receiver, so that when self.consumer.send() is used, the receiver is able to receive.
    pub fn register_consumer(&mut self) -> Receiver<ContentContainer<I>> {

        // Create new channel that carries ContentContainer values
        let (send, recv) = channel::<ContentContainer<I>>();
        self.consumer.replace(send);
        recv
    }

    /// Registers which callback function to call when you use self.callback
    pub fn register_callback(
        &mut self,
        function: Box<dyn FnMut(ContentContainer<I>) -> () + Send + 'static>,
    ) {
        self.call_back.replace(function);
    }

    /// Push window content clones to the callback and consumer
    pub fn flush(&mut self) {
        for (_, content) in &self.active_windows {
            if let Some(call_back) = &mut self.call_back {
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
    data: Mutex<Vec<ContentContainer<I>>>,
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
    fn start(&self, receiver: Receiver<ContentContainer<I>>) {
        let consumer_temp = self.inner.clone();
        thread::spawn(move || loop {
            match receiver.recv() { // .revc() is a blocking operation (wait until you get result or err)
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
            call_back: None,
            uri: "test_window".to_string()
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
    fn test_window_with_call_back() {
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
            call_back: None,
            uri: "test_window".to_string()
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