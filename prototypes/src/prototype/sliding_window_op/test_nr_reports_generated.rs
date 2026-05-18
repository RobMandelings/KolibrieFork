#[cfg(test)]
mod tests {
    use crate::prototype::event::Time;
    use crate::{Event, SliceStrategy, SlidingWindowOperator, WindowSnapshotStrategy};
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::prototype::helpers::{wc_by_params};
    use crate::prototype::slide_strategy::event_arrives::EventArrives;
    use crate::prototype::slide_strategy::slice_strategy::{SliceConsumer, SliceContainer};
    use crate::workloads::{create_events_for_workload, mk_workload};

    // Consumer that only counts how many reports it receives, for a single window.
    fn make_counting_consumer(counter: Rc<RefCell<usize>>)
                              -> SliceConsumer<String>
    {
        Box::new(move |_events: SliceContainer<String>| {
            *counter.borrow_mut() += 1;
        })
    }

    fn make_report_len_consumer(
        sink: Rc<RefCell<Vec<usize>>>,
    ) -> SliceConsumer<String> {
        Box::new(move |events| {
            sink.borrow_mut().push(events.0.len());
        })
    }

    // Factory for events with given timestamp and some dummy payload.
    fn mk_string_event(ts: Time) -> Event<String> {
        Event {
            ts,
            payload: format!("e{ts}"),
        }
    }

    #[test]
    fn window_size_5_slide_1_spread_1_reports_after_filling_and_then_each_step() {
        let w = wc_by_params(5,1);

        let strat: SliceStrategy<String> = SliceStrategy::new();
        let mut op = SlidingWindowOperator::new_default_iri(vec![w.clone()], strat);

        let report_count = Rc::new(RefCell::new(0usize));
        let consumer = make_counting_consumer(Rc::clone(&report_count));
        op.add_consumer(&w.window_iri, consumer);

        // 0,1,2...,9
        let workload = mk_workload(10, 1, 0);
        let events = create_events_for_workload(&workload, 0, mk_string_event);
        let ts: Vec<Time> = events.iter().map(|e| e.ts).collect();
        println!("{:?}", ts);

        for e in events {
            op.event_arrives(e);
        }

        let count = *report_count.borrow();
        assert_eq!(
            count, 4,
            "expected 4 reports for size=5, slide=1, 10 events with spread=1"
        );
    }

    #[test]
    fn window_size_5_slide_5_spread_1_reports_every_five_timestamps() {
        let w = wc_by_params(5, 5);
        let strat: SliceStrategy<String> = SliceStrategy::new();
        let mut op = SlidingWindowOperator::new_default_iri(vec![w.clone()], strat);

        let report_count = Rc::new(RefCell::new(0usize));
        let consumer = make_counting_consumer(Rc::clone(&report_count));
        op.add_consumer(&w.window_iri, consumer);

        // 0,1,2,...,11
        let workload = mk_workload(12, 1, 0);
        let events = create_events_for_workload(&workload, 0, mk_string_event);

        for e in events {
            op.event_arrives(e);
        }

        let count = *report_count.borrow();
        assert_eq!(
            count, 2,
            "expected 2 reports for size=5, slide=5, 11 events with spread=1"
        );
    }

    #[test]
    fn window_size_5_slide_5_spread_5_reports_with_single_event_each() {
        let w = wc_by_params(5, 5);

        let strat: SliceStrategy<String> = SliceStrategy::new();
        let mut op = SlidingWindowOperator::new_default_iri(vec![w.clone()], strat);

        let report_sizes = Rc::new(RefCell::new(Vec::<usize>::new()));
        let consumer = make_report_len_consumer(Rc::clone(&report_sizes));
        op.add_consumer(&w.window_iri, consumer);

        // Use spread=5 so timestamps = 6,11,16,21,26
        let workload = mk_workload(5, 5, 6);
        let events = create_events_for_workload(&workload, 0, mk_string_event);

        for e in events {
            op.event_arrives(e);
        }

        let report_sizes = report_sizes.borrow();
        assert_eq!(report_sizes.len(), 5, "expected 5 reports");

        // First report should contain zero events
        let expected = [0usize, 1, 1, 1, 1];

        assert_eq!(
            report_sizes.as_slice(),
            &expected,
            "unexpected event counts per report, got {:?}",
            *report_sizes
        );
    }
}