/*
 * Copyright © 2025 Volodymyr Khadzhaia
 * Copyright © 2025 Pieter Bonte
 * KU Leuven — Stream Intelligence Lab, Belgium
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::rsp::s2r::reporting::Report;
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

pub mod s2r_tests;
pub mod reporting;
pub mod sparql_window;

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

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct WindowTriple {
    pub s: String,
    pub p: String,
    pub o: String,
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