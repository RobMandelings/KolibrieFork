use crate::prototype::event::Time;
use crate::prototype::helpers::{wc};
use crate::prototype::slide_strategy::WindowSnapshotStrategy;
use crate::prototype::sliding_window_bounds::compute_earliest_open_time;
use crate::{make_string_event, CloneStrategy, Event, SliceStrategy, RcStrategy, SlidingWindowOperator, WindowParams};
use crate::prototype::slide_strategy::event_arrives::EventArrives;

/// Invariant here: events should be removed from content once all sliding window bounds' open times
/// are higher than the events' timestamp.

fn assert_event_respects_cutoff(e: &Event<String>, earliest: Time) {
    assert!(
        e.ts >= earliest,
        "found event with ts {} < earliest_open_time {}",
        e.ts,
        earliest
    );
}

fn assert_expire_respects_cutoff(op: &SlidingWindowOperator<String, SliceStrategy<String>>) {
    let earliest = compute_earliest_open_time(op.sliding_windows.values());

    for e in &op.strategy.content {
        assert_event_respects_cutoff(e, earliest);
    }
}

fn assert_clone_respects_cutoff(op: &SlidingWindowOperator<String, CloneStrategy<String>>) {
    let earliest = compute_earliest_open_time(op.sliding_windows.values());

    for e in &op.strategy.content {
        assert_event_respects_cutoff(e, earliest);
    }
}

fn assert_rc_respects_cutoff(op: &SlidingWindowOperator<String, RcStrategy<String>>) {
    let earliest = compute_earliest_open_time(op.sliding_windows.values());

    for e in &op.strategy.content {
        assert_event_respects_cutoff(e, earliest);
    }
}

#[test]
fn strategies_invariant_no_expiry() {
    let config = wc(10, 10, 0);

    fn drive_no_expiry_scenario<S>(op: &mut SlidingWindowOperator<String, S>)
    where
        S: WindowSnapshotStrategy<String>,
    {
        op.event_arrives(make_string_event(0));
        op.event_arrives(make_string_event(1));
        op.event_arrives(make_string_event(2));
    }

    // Expire
    {
        let mut op = SlidingWindowOperator::single_window(config.clone(), SliceStrategy::new());
        drive_no_expiry_scenario(&mut op);
        assert_expire_respects_cutoff(&op);
    }

    // Clone
    {
        let mut op = SlidingWindowOperator::single_window(config.clone(), CloneStrategy::new());
        drive_no_expiry_scenario(&mut op);
        assert_clone_respects_cutoff(&op);
    }

    // RefCount
    {
        let mut op = SlidingWindowOperator::single_window(config, RcStrategy::new());
        drive_no_expiry_scenario(&mut op);
        assert_rc_respects_cutoff(&op);
    }
}

#[test]
fn strategies_drop_events_before_cutoff() {
    let config = wc(10, 10, 0);

    fn drive_expiry_scenario<S>(op: &mut SlidingWindowOperator<String, S>)
    where
        S: WindowSnapshotStrategy<String>,
    {
        // Fill first window [0, 10)
        op.event_arrives(make_string_event(0));
        op.event_arrives(make_string_event(5));
        op.event_arrives(make_string_event(9));
        // Move into second window [10, 20)
        op.event_arrives(make_string_event(11));
        op.event_arrives(make_string_event(15));
    }

    // Expire
    {
        let mut op = SlidingWindowOperator::single_window(config.clone(), SliceStrategy::new());
        drive_expiry_scenario(&mut op);
        assert_expire_respects_cutoff(&op);
    }

    // Clone
    {
        let mut op = SlidingWindowOperator::single_window(config.clone(), CloneStrategy::new());
        drive_expiry_scenario(&mut op);
        assert_clone_respects_cutoff(&op);
    }

    // RefCount
    {
        let mut op = SlidingWindowOperator::single_window(config, RcStrategy::new());
        drive_expiry_scenario(&mut op);
        assert_rc_respects_cutoff(&op);
    }
}

#[test]
fn overlapping_windows_earliest_from_second_window() {
    fn drive_overlapping_scenario<S>(op: &mut SlidingWindowOperator<String, S>)
    where
        S: WindowSnapshotStrategy<String>,
    {
        // Event at 4: belongs to the first window
        op.event_arrives(make_string_event(4));

        // Event at 9: in both windows.
        op.event_arrives(make_string_event(9));

        // Event at 11: this will cause window 1 to slide to (11,20],
        // but window 2 stays at (5,15], so earliest_open_time is 5.
        // 9 is not in window 1 anymore, but it is in window 2, so keep it
        // 4 is neither in window 1 nor in window 2, so this can be dropped
        op.event_arrives(make_string_event(10));

        // Another event inside both windows.
        op.event_arrives(make_string_event(12));
    }

    let expected_content: Vec<Time> = vec![9, 10, 12];

    // Window 1: [0,10), [10,20), ...
    let wc1 = wc(10, 10, 0);

    // Window 2: [5,15), [15,25), ...
    let wc2 = wc(10, 10, 5);

    // ExpireStrategy
    {
        let mut op: SlidingWindowOperator<String, SliceStrategy<String>> =
            SlidingWindowOperator::new_default_iri(vec![wc1.clone(), wc2.clone()], SliceStrategy::new());

        drive_overlapping_scenario(&mut op);
        let timestamps: Vec<Time> = op.strategy.content.iter().map(|e| e.ts).collect();
        assert_eq!(timestamps, expected_content);
    }

    // CloneStrategy
    {
        let strategy: CloneStrategy<String> = CloneStrategy::new();
        let mut op = SlidingWindowOperator::new_default_iri(vec![wc1.clone(), wc2.clone()], strategy);
        drive_overlapping_scenario(&mut op);
        let timestamps: Vec<Time> = op.strategy.content.iter().map(|e| e.ts).collect();
        assert_eq!(timestamps, expected_content);
    }

    // RefCountStrategy
    {
        let strategy: RcStrategy<String> = RcStrategy::new();
        let mut op = SlidingWindowOperator::new_default_iri(vec![wc1, wc2], strategy);
        drive_overlapping_scenario(&mut op);
        let timestamps: Vec<Time> = op.strategy.content.iter().map(|e| e.ts).collect();
        assert_eq!(timestamps, expected_content);
    }
}
