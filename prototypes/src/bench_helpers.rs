use std::fmt::Debug;
use std::hash::Hash;
// TODO find a way to put the criterion lib under dev-dependencies, not normal deps
use crate::prototype::slide_strategy::arc_strategy::ArcContainer;
use crate::prototype::slide_strategy::clone_strategy::CloneContainer;
use crate::prototype::slide_strategy::expire_strategy::SliceContainer;
use crate::prototype::slide_strategy::rc_strategy::RcContainer;
use crate::s2r::{ContentContainer, LegacyWindow, Report, ReportStrategy, Tick};
use crate::workloads::Workload;
use crate::{ArcStrategy, CloneStrategy, Event, ExpireStrategy, RcStrategy
    , SlidingWindowOperator, WindowSnapshotStrategy,
};
use criterion::black_box;

pub type Time = u64;

fn run_legacy_bench<I>(
    windows: &mut Vec<LegacyWindow<I>>,
    events: Vec<Event<I>>,
)
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    for e in events.into_iter() {
        for window in windows.iter_mut() {
            window.add_to_window(e.payload.clone(), e.ts as usize);
        }
    }
}

fn run_throughput_bench<I, S>(
    op: &mut SlidingWindowOperator<I, S>,
    events: Vec<Event<I>>,
)
where
    S: WindowSnapshotStrategy<I>,
{
    for e in events.into_iter() {
        op.event_arrives(black_box(e));
    }
}

pub fn run_strategy_legacy<I>(
    workload: &Workload,
    events: Vec<Event<I>>,
)
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    let consume = Box::new(|_events: ContentContainer<I>| {});

    let mut windows = vec![];
    let strats = vec![ReportStrategy::OnWindowClose];
    for window_config in &workload.windows {
        let report = Report::new_with_strats(strats.clone());
        let mut window = LegacyWindow::new(
            window_config.window_params.size as usize,
            window_config.window_params.slide as usize,
            report,
            Tick::TimeDriven,
            window_config.window_iri.to_string(),
        );

        window.register_callback(consume.clone());
        windows.push(window);
    }

    run_legacy_bench(&mut windows, events);
}

pub fn run_strategy_expire<I>(
    workload: &Workload,
    events: Vec<Event<I>>,
)
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    let consume = Box::new(|_events: SliceContainer<I>| {});
    let strat = ExpireStrategy::new();
    let mut op = SlidingWindowOperator::new_default_iri(workload.windows.clone(), strat);
    for window_config in &workload.windows {
        let window_iri = &window_config.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    run_throughput_bench(&mut op, events);
}

pub type EventFactory<I> = fn(Time) -> Event<I>;

pub fn run_strategy_clone<I>(
    workload: &Workload,
    events: Vec<Event<I>>
)
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    let consume = Box::new(|_events: CloneContainer<I>| {});
    let strat = CloneStrategy::new();
    let mut op = SlidingWindowOperator::new_default_iri(workload.windows.clone(), strat);
    for window_config in &workload.windows {
        let window_iri = &window_config.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    run_throughput_bench(&mut op, events);
}

pub fn run_strategy_arc<I>(
    workload: &Workload,
    events: Vec<Event<I>>
)
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    let consume = Box::new(|_events: ArcContainer<I>| {});
    let strat = ArcStrategy::new();
    let mut op = SlidingWindowOperator::new_default_iri(workload.windows.clone(), strat);
    for window_config in &workload.windows {
        let window_iri = &window_config.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    run_throughput_bench(&mut op, events);
}

pub fn run_strategy_rc<I>(
    workload: &Workload,
    events: Vec<Event<I>>
)
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    let consume = Box::new(|_events: RcContainer<I>| {});
    let strat = RcStrategy::new();
    let mut op = SlidingWindowOperator::new_default_iri(workload.windows.clone(), strat);
    for window_config in &workload.windows {
        let window_iri = &window_config.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    run_throughput_bench(&mut op, events);
}