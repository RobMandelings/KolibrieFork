use std::rc::Rc;
use std::sync::Arc;
// TODO find a way to put the criterion lib under dev-dependencies, not normal deps
use crate::workloads::Workload;
use crate::{
    ArcStrategy, CloneStrategy, Event, ExpireStrategy, RcStrategy, SlidingWindowOperator,
    WindowParams, WindowSnapshotStrategy, event,
};
use criterion::black_box;
use crate::prototype::slide_strategy::arc_strategy::ArcContainer;
use crate::prototype::slide_strategy::clone_strategy::CloneContainer;
use crate::prototype::slide_strategy::expire_strategy::SliceContainer;
use crate::prototype::slide_strategy::rc_strategy::RcContainer;

pub type Time = u64;

fn run_throughput_bench<S>(op: &mut SlidingWindowOperator<String, S>, nr_events: usize)
where
    S: WindowSnapshotStrategy<String>,
{
    let max_ts: crate::prototype::event::Time = nr_events as u64;

    // Generate events with timestamps ranging 0, 1, 2,... (not overlapping, constant distance between two events)
    // Maybe later customise?
    let events: Vec<Event<String>> = (0..max_ts).map(|i| event(i)).collect();

    for e in events.into_iter() {
        op.event_arrives(black_box(e));
    }
}

pub fn run_strategy_expire(workload: &Workload) {
    let consume = Box::new(|_events: SliceContainer<String>| {});
    let strat = ExpireStrategy::new();
    let mut op = SlidingWindowOperator::new_default_iri(workload.windows.clone(), strat);
    for window in &workload.windows {
        let window_iri = &window.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    run_throughput_bench(&mut op, workload.nr_events);
}

pub fn run_strategy_clone(workload: &Workload) {
    let consume = Box::new(|_events: CloneContainer<String>| {});
    let strat = CloneStrategy::new();
    let mut op = SlidingWindowOperator::new_default_iri(workload.windows.clone(), strat);
    for window in &workload.windows {
        let window_iri = &window.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    run_throughput_bench(&mut op, workload.nr_events);
}

pub fn run_strategy_arc(workload: &Workload) {
    let consume = Box::new(|_events: ArcContainer<String>| {});
    let strat = ArcStrategy::new();
    let mut op = SlidingWindowOperator::new_default_iri(workload.windows.clone(), strat);
    for window in &workload.windows {
        let window_iri = &window.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    run_throughput_bench(&mut op, workload.nr_events);
}

pub fn run_strategy_refcount(workload: &Workload) {
    let consume = Box::new(|_events: RcContainer<String>| {});
    let strat = RcStrategy::new();
    let mut op = SlidingWindowOperator::new_default_iri(workload.windows.clone(), strat);
    for window in &workload.windows {
        let window_iri = &window.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    run_throughput_bench(&mut op, workload.nr_events);
}
