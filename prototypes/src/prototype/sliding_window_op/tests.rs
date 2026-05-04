use super::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::{make_string_event, Event, SliceStrategy, WindowParams};
use crate::prototype::event::Time;
use crate::prototype::helpers::{wc};
use crate::prototype::slide_strategy::slice_strategy::SliceContainer;

#[test]
fn sliding_window_operator_reports_on_window_close() {
    let config = wc(10, 10, 0);
    let window_iri = config.window_iri.clone();

    // Shared buffer to capture what the Consumer sees.
    let reported: Rc<RefCell<Vec<Vec<Time>>>> = Rc::new(RefCell::new(Vec::new()));
    let reported_clone = Rc::clone(&reported);

    // Consumer that records timestamps of reported events.
    let consume_fn = Box::new(move |events: SliceContainer<String>| {
        let ts_list: Vec<Time> = events.0.iter().map(|e| e.ts).collect();
        reported_clone.borrow_mut().push(ts_list);
    });
    let mut strat = SliceStrategy::new();
    let mut op = SlidingWindowOperator::single_window(config, strat);
    op.add_consumer(&window_iri, consume_fn);

    // First window (0,10]: events at 1, 3, 7.
    op.event_arrives(make_string_event(1));
    op.event_arrives(make_string_event(3));
    op.event_arrives(make_string_event(7));

    // Event at ts = 11 closes (0,10] and should report [1,3,7].
    op.event_arrives(make_string_event(11));

    // Second window (10,20]: events at 13, 19.
    op.event_arrives(make_string_event(13));
    op.event_arrives(make_string_event(19));

    // Event at ts = 21 closes (10,20] and should report [11, 13,19].
    op.event_arrives(make_string_event(21));

    let reports = reported.borrow();
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0], vec![1, 3, 7]);
    assert_eq!(reports[1], vec![11, 13, 19]);
}

#[test]
fn sliding_window_operator_reports_overlapping_windows_secret_intervals() {
    let config = wc(10, 5, 0);
    let window_iri = config.window_iri.clone();

    let reported: Rc<RefCell<Vec<Vec<Time>>>> = Rc::new(RefCell::new(Vec::new()));
    let reported_clone = Rc::clone(&reported);

    let consume_fn = Box::new(move |events: SliceContainer<String>| {
        let ts_list: Vec<Time> = events.0.iter().map(|e| e.ts).collect();
        reported_clone.borrow_mut().push(ts_list);
    });
    let mut strat = SliceStrategy::new();
    let mut op = SlidingWindowOperator::single_window(config, strat);
    op.add_consumer(&window_iri, consume_fn);

    // Windows under (open, close] semantics:
    // W1: (0, 10]
    // W2: (5, 15]
    // W3: (10, 20]

    // All three events go into W1
    op.event_arrives(make_string_event(3));
    op.event_arrives(make_string_event(7));
    op.event_arrives(make_string_event(10));

    // Close W1 at ts = 10; So ts = 11 closes W1 report W1's content.
    // ts 7 and 10 are included in W2
    // ts 11, 14 as well
    op.event_arrives(make_string_event(11));
    op.event_arrives(make_string_event(14));
    // Close W2 at ts = 15; report W2's content.
    op.event_arrives(make_string_event(16));
    // Close W3 at ts = 20; report W3's content.
    op.event_arrives(make_string_event(21));

    let reports = reported.borrow();

    // We still expect three reports, one per window.
    assert_eq!(reports.len(), 3);

    // Under (open, close] semantics and the above event times:
    // W1: (0,10]  => [7, 10]          (3 is outside, 7 and 10 inside)
    // W2: (5,15]  => [11, 14, 15]     (7,10 ≤ 5 is false; 11,14,15 inside)
    // W3: (10,20] => [20]             (11,14 ≤ 10 is false; only 20 inside)
    assert_eq!(reports[0], vec![3, 7, 10]);
    assert_eq!(reports[1], vec![7, 10, 11, 14]);
    assert_eq!(reports[2], vec![11, 14, 16]);
}