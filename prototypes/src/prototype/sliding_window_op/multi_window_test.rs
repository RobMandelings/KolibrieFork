use crate::prototype::event::Time;
use crate::prototype::helpers::{wc};
use crate::prototype::slide_strategy::expire_strategy::SliceContainer;
use crate::{make_string_event, ExpireStrategy, SlidingWindowOperator, WindowSnapshotStrategy};
use std::cell::RefCell;
use std::rc::Rc;

fn make_consumer_for_window(
    reported: Rc<RefCell<Vec<Vec<Vec<Time>>>>>,
    window_index: usize,
) -> Box<dyn for<'a> Fn(SliceContainer<String>)> {
    Box::new(move |events: SliceContainer<String>| {
        let ts_list: Vec<Time> = events.0.iter().map(|e| e.ts).collect();
        reported.borrow_mut()[window_index].push(ts_list);
    })
}

#[test]
fn sliding_window_operator_reports_multiple_windows_with_distinct_params() {
    // Window 0: size 10, slide 10, offset 0 -> (0,10], (10,20], ...
    let w0 = wc(10, 10, 0);
    // Window 1: size 6, slide 4, offset 0 -> (0,6], (4,10], (8,14], ...
    let w1 = wc(6,4,0);

    // reported[0] = reports from window 0
    // reported[1] = reports from window 1
    let reported: Rc<RefCell<Vec<Vec<Vec<Time>>>>> = Rc::new(RefCell::new(vec![Vec::new(), Vec::new()]));

    let consume_w0 = make_consumer_for_window(Rc::clone(&reported), 0);
    let consume_w1 = make_consumer_for_window(Rc::clone(&reported), 1);
    let consumers = vec![
        (w0.window_iri.clone(), consume_w0),
        (w1.window_iri.clone(), consume_w1)
    ];

    let strat: ExpireStrategy<String> = ExpireStrategy::new();
    // Two windows in one operator
    let mut op = SlidingWindowOperator::new_default_iri(vec![w0, w1], strat);
    for (iri, consumer) in consumers {
        op.add_consumer(&iri, consumer);
    }

    // Stream of events
    // t=2,5,7,9,11,13,17
    op.event_arrives(make_string_event(2));
    op.event_arrives(make_string_event(5));
    op.event_arrives(make_string_event(7));
    op.event_arrives(make_string_event(9));
    op.event_arrives(make_string_event(11));
    op.event_arrives(make_string_event(13));
    op.event_arrives(make_string_event(17));

    let reports = reported.borrow();

    // Assert we have some reports for both windows
    // (exact expectations depend on your precise (open, close] semantics and slide implementation)
    assert_eq!(reports[0].len(), 1, "window 0 should produce 1 report");
    assert_eq!(reports[1].len(), 3, "window 1 should produce 3 reports");

    assert_eq!(reports[0][0], vec![2, 5, 7, 9]);

    // Reports for window 1
    assert_eq!(reports[1][0], vec![2, 5]);
    assert_eq!(reports[1][1], vec![5, 7, 9]);
    assert_eq!(reports[1][2], vec![9, 11, 13]);
}