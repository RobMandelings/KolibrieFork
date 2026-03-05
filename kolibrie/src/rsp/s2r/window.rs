use std::collections::hash_set::{IntoIter, Iter};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::mem;

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct WindowTriple {
    pub s: String,
    pub p: String,
    pub o: String,
}

/// Window is represented as an opening timestamp and closing timestamp
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct WindowBounds {
    pub(crate) open: usize,  // timestamp for when the window is opened
    pub(crate) close: usize, // timestamp for when the window is closed
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
    pub(crate) fn new() -> WindowContent<I> {
        WindowContent {
            elements: HashSet::new(),
            last_timestamp_changed: 0,
            origin: String::default(),
        }
    }
    pub(crate) fn new_with_origin(origin: &str) -> WindowContent<I> {
        WindowContent {
            elements: HashSet::new(),
            last_timestamp_changed: 0,
            origin: origin.to_string(),
        }
    }
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    pub(crate) fn add(&mut self, triple: I, ts: usize) {
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