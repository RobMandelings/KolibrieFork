use crate::prototype::event::Time;
use crate::prototype::slide_strategy::{ItemsReport, WindowSnapshotStrategy};
use crate::prototype::window_bounds::after_open;
use crate::Event;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::iter::Chain;
use std::slice::Iter;

/// Concrete slide_strategy: expire old events efficiently from the front,
/// report the active window as up to two slices from a VecDeque.
pub struct SliceDequeStrategy<I: Clone> {
    // Outer Vec: one entry per window
    // Inner Vec: consumers for that window
    consume_fns: HashMap<String, Vec<RefCell<Box<dyn FnMut(SliceDequeContainer<I>)>>>>,
    pub content: VecDeque<Event<I>>,
}

impl<I: Clone> SliceDequeStrategy<I> {
    /// Return the logical suffix of the deque containing all events with ts after the open time.
    /// Because VecDeque is a ring buffer, the suffix may consist of two slices.
    pub fn slice_by_ts(&self, open: Time) -> SliceDequeContainer<'_, I> {
        let start = self
            .content
            .iter()
            .position(|e| after_open(&open, &e.ts))
            .unwrap_or(self.content.len());

        let (front, back) = self.content.as_slices();
        let front_len = front.len();

        let (first, second) = if start < front_len {
            (&front[start..], back)
        } else {
            let offset = start - front_len;
            (&back[offset.min(back.len())..], &[][..])
        };

        SliceDequeContainer(first, second)
    }
}

pub type SliceDequeConsumer<I> = Box<dyn Fn(SliceDequeContainer<I>)>;

pub struct SliceDequeContainer<'a, I>(pub &'a [Event<I>], pub &'a [Event<I>]);

impl<I> Clone for SliceDequeContainer<'_, I> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<'a, I> SliceDequeContainer<'a, I> {
    pub fn iter(&self) -> Chain<Iter<'a, Event<I>>, Iter<'a, Event<I>>> {
        self.0.iter().chain(self.1.iter())
    }

    pub fn len(&self) -> usize {
        self.0.len() + self.1.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty() && self.1.is_empty()
    }
}

impl<I: 'static> ItemsReport<I> for SliceDequeContainer<'_, I> {
    fn get_last_timestamp_changed(&self) -> Time {
        self.1
            .last()
            .or_else(|| self.0.last())
            .expect("SliceDequeContainer must be non-empty")
            .ts
    }

    fn iter_items(&self) -> impl Iterator<Item = &I> {
        self.iter().map(|e| &e.payload)
    }
}

impl<I: Clone + 'static> WindowSnapshotStrategy<I> for SliceDequeStrategy<I> {
    type ReportType<'a> = SliceDequeContainer<'a, I>;

    fn new() -> Self {
        Self {
            consume_fns: HashMap::new(),
            content: VecDeque::new(),
        }
    }

    fn with_capacity(reserve: usize) -> Self {
        Self {
            consume_fns: HashMap::new(),
            content: VecDeque::with_capacity(reserve),
        }
    }

    fn report_window<'a>(&mut self, window_iri: &str, open_time: Time) {
        let slice = self.slice_by_ts(open_time);
        self.consume_window(window_iri, slice);
    }

    fn drop_expired_events(&mut self, open_time: Time) {
        while let Some(front) = self.content.front() {
            if after_open(&open_time, &front.ts) {
                break;
            }
            self.content.pop_front();
        }
    }

    fn add_event(&mut self, event: Event<I>) {
        self.content.push_back(event);
    }

    fn consume_fns(
        &self,
    ) -> &HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>> {
        &self.consume_fns
    }

    fn consume_fns_mut(
        &mut self,
    ) -> &mut HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>> {
        &mut self.consume_fns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::make_string_event;
    use std::collections::VecDeque;

    fn consume_fn() -> Box<dyn for<'a> Fn(SliceDequeContainer<String>)> {
        Box::new(|_| {})
    }

    #[test]
    fn expire_events_none_expired() {
        let mut wc: SliceDequeStrategy<String> = SliceDequeStrategy::new();

        wc.content = VecDeque::from(vec![
            make_string_event(10),
            make_string_event(20),
            make_string_event(30),
        ]);
        wc.add_consumer("0", consume_fn());

        let slice = wc.slice_by_ts(5);
        assert_eq!(slice.len(), 3);

        let mut it = slice.iter();
        assert_eq!(it.next().unwrap().ts, 10);
        assert_eq!(it.next().unwrap().ts, 20);
        assert_eq!(it.next().unwrap().ts, 30);
        assert!(it.next().is_none());

        wc.drop_expired_events(5);
        assert_eq!(wc.content.len(), 3);
    }

    #[test]
    fn expire_events_some_expired() {
        let mut wc: SliceDequeStrategy<String> = SliceDequeStrategy::new();

        wc.content = VecDeque::from(vec![
            make_string_event(10),
            make_string_event(20),
            make_string_event(30),
        ]);
        wc.add_consumer("0", consume_fn());

        let slice = wc.slice_by_ts(25);
        assert_eq!(slice.len(), 1);

        let collected: Vec<_> = slice.iter().map(|e| e.ts).collect();
        assert_eq!(collected, vec![30]);

        wc.drop_expired_events(25);
        assert_eq!(wc.content.len(), 1);
        assert_eq!(wc.content.front().unwrap().ts, 30);
    }

    #[test]
    fn expire_events_all_expired() {
        let mut wc: SliceDequeStrategy<String> = SliceDequeStrategy::new();

        wc.content = VecDeque::from(vec![
            make_string_event(10),
            make_string_event(20),
            make_string_event(30),
        ]);
        wc.add_consumer("0", consume_fn());

        let slice = wc.slice_by_ts(100);
        assert_eq!(slice.len(), 0);
        assert!(slice.is_empty());

        wc.drop_expired_events(100);
        assert_eq!(wc.content.len(), 0);
    }

    #[test]
    fn wrapped_two_slice_iteration_stays_in_order() {
        let mut wc: SliceDequeStrategy<String> = SliceDequeStrategy::with_capacity(8);

        wc.add_event(make_string_event(10));
        wc.add_event(make_string_event(20));
        wc.add_event(make_string_event(30));
        wc.add_event(make_string_event(40));

        wc.drop_expired_events(25); // removes 10,20
        wc.add_event(make_string_event(50));
        wc.add_event(make_string_event(60));
        wc.add_event(make_string_event(70));

        let slice = wc.slice_by_ts(35);
        let collected: Vec<_> = slice.iter().map(|e| e.ts).collect();

        assert_eq!(collected, vec![40, 50, 60, 70]);
    }
}