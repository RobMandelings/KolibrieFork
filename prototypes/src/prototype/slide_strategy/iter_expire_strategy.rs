// use crate::prototype::event::Time;
// use crate::prototype::slide_strategy::{CutoffOrOpen, ItemsReport, WindowSnapshotStrategy};
// use crate::{Event, IRI};
// use log::debug;
// use std::cell::RefCell;
// use std::collections::{BTreeMap, HashMap};
// use std::rc::Rc;
//
// // TODO use generics for better performance?
// #[derive(Clone)]
// pub struct IterExpireContainer<'a, I>
// where
//     I: 'static,
// {
//     pub last_timestamp_changed: Time,
//     // TODO ideal data structure?
//     batches: &'a BTreeMap<Time, Vec<I>>,
//     open: Time,
// }
//
// pub type IterConsumer<I> = dyn for<'a> FnMut(IterExpireContainer<'a, I>);
//
// /// Continuous report: containing slice
// pub type IterReport<'a, I> = <IterExpireStrategy<I> as WindowSnapshotStrategy<I>>::ReportType<'a>;
//
// // for any <'a, I, It>, define the methods for the type IterExpireContainer<'a, I, It>
// impl<'a, I> IterExpireContainer<'a, I> {
//     pub fn new(
//         last_ts: Time,
//         batches: &'a BTreeMap<Time, Vec<I>>,
//         open: Time,
//     ) -> IterExpireContainer<'a, I> {
//         IterExpireContainer {
//             last_timestamp_changed: last_ts,
//             batches,
//             open,
//         }
//     }
// }
//
// impl<I> ItemsReport<I> for IterExpireContainer<'_, I> {
//     fn get_last_timestamp_changed(&self) -> Time {
//         self.last_timestamp_changed
//     }
//
//     fn iter_items(&self) -> impl Iterator<Item = &I> {
//         self.batches
//             .range(self.open..)
//             .flat_map(|(_, batch)| batch.iter())
//     }
// }
//
// // pub struct IterExpire<'a, I> {
// //     inner: std::iter::FlatMap<
// //         std::collections::btree_map::Range<'a, Time, Vec<I>>,
// //         std::slice::Iter<'a, I>,
// //         fn((&'a Time, &'a Vec<I>)) -> std::slice::Iter<'a, I>>,
// // }
// //
// // impl<'a, I> IterExpire<'a, I> {
// //     fn new(batches: &'a BTreeMap<Time, Vec<I>>, open: Time) -> Self {
// //         IterExpire {
// //             inner: batches
// //                 .range(open..)
// //                 .flat_map(|(_, batch)| batch.iter()),
// //         }
// //     }
// // }
//
// // impl<'a, I> Iterator for IterExpire<'a, I> {
// //     type Item = &'a I;
// //     fn next(&mut self) -> Option<Self::Item> {
// //         self.inner.next()
// //     }
// // }
//
// // impl<'a, I> IntoIterator for &'a IterExpireContainer<'a, I> {
// //     type Item = &'a I;
// //     type IntoIter = IterExpire<'a, I>;
// //
// //     fn into_iter(self) -> Self::IntoIter {
// //         IterExpire::new(self.batches, self.open)
// //     }
// // }
//
// /// Concrete slide_strategy: expire old events, report them as owned Events.
// pub struct IterExpireStrategy<I: Clone + 'static> {
//     // Outer Vec: one entry per window
//     // Inner Vec: consumers for that window
//     consume_fns: HashMap<String, Vec<RefCell<Box<dyn FnMut(IterExpireContainer<I>)>>>>,
//     pub batches: BTreeMap<Time, Vec<I>>,
// }
//
// impl<I: Clone> IterExpireStrategy<I> {
//     fn get_last_batch_ts(&self) -> Time {
//         self.batches.last_key_value().map(|(&t, _)| t).unwrap()
//     }
// }
//
// // TODO what if you allow references to be passed instead then? So it contains references?
// // That would be weird
// impl<I: Clone + 'static> WindowSnapshotStrategy<I> for IterExpireStrategy<I> {
//     type ReportType<'a> = IterExpireContainer<'a, I>;
//
//     fn new() -> Self {
//         Self {
//             consume_fns: HashMap::new(),
//             batches: BTreeMap::new(),
//         }
//     }
//
//     fn report_window<'a>(&mut self, window_iri: &str, open_time: Time) {
//         let container =
//             IterExpireContainer::new(self.get_last_batch_ts(), &self.batches, open_time);
//         self.consume_window(window_iri, container);
//     }
//
//     fn drop_expired_events(&mut self, open_time: Time) {
//         let newer = self.batches.split_off(&open_time);
//         self.batches = newer;
//     }
//
//     fn add_event(&mut self, event: Event<I>) {
//         self.batches
//             .entry(event.ts)
//             .or_insert_with(Vec::new)
//             .push(event.payload);
//     }
//
//     fn consume_fns(
//         &self,
//     ) -> &HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>> {
//         &self.consume_fns
//     }
//
//     fn consume_fns_mut(
//         &mut self,
//     ) -> &mut HashMap<String, Vec<RefCell<Box<dyn for<'a> FnMut(Self::ReportType<'a>)>>>> {
//         &mut self.consume_fns
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::prototype::slide_strategy::CutoffOrOpen::Open;
//
//     fn consume_fn() -> Box<IterConsumer<String>> {
//         Box::new(|_| {})
//     }
//
//     fn create_events() -> Vec<Event<String>> {
//         let events = vec![
//             Event::new(10, "A".to_string()),
//             Event::new(20, "B".to_string()),
//             Event::new(30, "C".to_string()),
//         ];
//         events
//     }
//
//     fn init_expire() -> IterExpireStrategy<String> {
//         let mut strat: IterExpireStrategy<String> = IterExpireStrategy::new();
//         let events = create_events();
//         for e in events {
//             strat.add_event(e);
//         }
//
//         strat
//     }
//
//     fn init_expire_empty_consume() -> IterExpireStrategy<String> {
//         let mut strat = init_expire();
//         strat.add_consumer("0", consume_fn());
//         strat
//     }
//
//     fn get_slice(strat: &IterExpireStrategy<String>, open: Time) -> IterExpireContainer<String> {
//         IterExpireContainer::new(strat.get_last_batch_ts(), &strat.batches, open)
//     }
//
//     #[test]
//     fn expire_events_none_expired() {
//         let strat = init_expire_empty_consume();
//         let slice = get_slice(&strat, 5);
//         let iter = slice.iter_items();
//         let vec: Vec<&String> = iter.collect();
//         assert_eq!(vec.len(), 3);
//     }
//
//     #[test]
//     fn expire_events_some_expired() {
//         let strat = init_expire_empty_consume();
//         let slice = get_slice(&strat, 25);
//         let iter = slice.iter_items();
//         let vec: Vec<&String> = iter.collect();
//
//         println!("{}", vec[0]);
//         assert_eq!(vec.len(), 1);
//         assert_eq!(vec[0], "C");
//     }
//
//     #[test]
//     fn expire_events_all_expired() {
//         let strat = init_expire_empty_consume();
//         let slice = get_slice(&strat, 100);
//         let iter = slice.iter_items();
//         let vec: Vec<&String> = iter.collect();
//         assert_eq!(vec.len(), 0);
//     }
//
//     #[test]
//     fn test_consumer() {
//         let mut strat = init_expire();
//         let consumer = Box::new(|report: IterReport<String>| {
//             println!("{}", report.last_timestamp_changed);
//             println!(
//                 "{}",
//                 report
//                     .iter_items()
//                     .map(|s: &String| s.as_str())
//                     .collect::<Vec<&str>>()
//                     .join(",")
//             );
//         });
//         strat.add_consumer("0", consumer);
//         strat.report_window("0", 20);
//     }
// }
