use crate::prototype::helpers::construct_window_configs;
// TODO find a way to put the criterion lib under dev-dependencies, not normal deps
use crate::prototype::slide_strategy::arc_strategy::ArcContainer;
use crate::prototype::slide_strategy::clone_strategy::CloneContainer;
use crate::prototype::slide_strategy::slice_strategy::SliceContainer;
use crate::prototype::slide_strategy::rc_strategy::RcContainer;
use crate::s2r::{ContentContainer, LegacyWindow, Report, ReportStrategy, Tick};
use crate::workloads::{create_events_for_workload, Workload};
use crate::{ArcStrategy, CloneStrategy, Event, SliceStrategy, RcStrategy
            , SlidingWindowOperator, WindowSnapshotStrategy,
};
use criterion::black_box;
use std::fmt::Debug;
use std::hash::Hash;

pub type Time = u64;

/// Lets events arrive to run the legacy S2R operator
fn run_legacy<I>(
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

/// Lets events arrive to run the S2R operator
pub fn run_new<I, S>(
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

fn build_legacy_windows<I>(workload: &Workload) -> Vec<LegacyWindow<I>>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    let consume = Box::new(|_events: ContentContainer<I>| {});
    let strats = vec![ReportStrategy::OnWindowClose];
    let window_configs = construct_window_configs(workload);

    let mut windows = Vec::with_capacity(window_configs.len());

    for window_config in &window_configs {
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

    windows
}

fn build_operator_slice<I>(workload: &Workload) -> SlidingWindowOperator<I, SliceStrategy<I>>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    let consume = Box::new(|_events: SliceContainer<I>| {});
    let strat = SliceStrategy::new();

    let window_configs = construct_window_configs(workload);
    let mut op = SlidingWindowOperator::new_default_iri(window_configs.clone(), strat);

    for window_config in &window_configs {
        let window_iri = &window_config.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    op
}

fn build_operator_clone<I>(workload: &Workload) -> SlidingWindowOperator<I, CloneStrategy<I>>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    let consume = Box::new(|_events: CloneContainer<I>| {});
    let strat = CloneStrategy::new();

    let window_configs = construct_window_configs(workload);
    let mut op = SlidingWindowOperator::new_default_iri(window_configs.clone(), strat);

    for window_config in &window_configs {
        let window_iri = &window_config.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    op
}

fn build_operator_arc<I>(workload: &Workload) -> SlidingWindowOperator<I, ArcStrategy<I>>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    let consume = Box::new(|_events: ArcContainer<I>| {});
    let strat = ArcStrategy::new();

    let window_configs = construct_window_configs(workload);
    let mut op = SlidingWindowOperator::new_default_iri(window_configs.clone(), strat);

    for window_config in &window_configs {
        let window_iri = &window_config.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    op
}

fn build_operator_rc<I>(workload: &Workload) -> SlidingWindowOperator<I, RcStrategy<I>>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    let consume = Box::new(|_events: RcContainer<I>| {});
    let strat = RcStrategy::new();

    let window_configs = construct_window_configs(workload);
    let mut op = SlidingWindowOperator::new_default_iri(window_configs.clone(), strat);

    for window_config in &window_configs {
        let window_iri = &window_config.window_iri;
        op.add_consumer(window_iri, consume.clone());
    }

    op
}


pub type RunnerFactory = Box<dyn Fn() -> Box<dyn FnOnce()>>;

pub fn create_legacy_factory<I, F>(
    workload: &Workload,
    event_factory: F,
) -> RunnerFactory
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    F: Fn(Time) -> Event<I> + Clone + 'static,
{
    let workload = workload.clone();

    Box::new(move || {
        let events = create_events_for_workload(&workload, &event_factory);
        let mut windows = build_legacy_windows(&workload);

        Box::new(move || {
            run_legacy(&mut windows, events);
        })
    })
}

pub fn create_slice_factory<I, F>(
    workload: &Workload,
    event_factory: F,
) -> RunnerFactory
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    F: Fn(Time) -> Event<I> + Clone + 'static,
{
    let workload = workload.clone();

    Box::new(move || {
        let events = create_events_for_workload(&workload, &event_factory);
        let mut op = build_operator_slice(&workload);

        Box::new(move || {
            run_new(&mut op, events);
        })
    })
}

pub type EventFactory<I> = Box<dyn Fn(Time) -> Event<I>>;

pub fn create_clone_factory<I, F>(
    workload: &Workload,
    event_factory: F,
) -> RunnerFactory
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    F: Fn(Time) -> Event<I> + Clone + 'static,
{
    let workload = workload.clone();

    Box::new(move || {
        let events = create_events_for_workload(&workload, &event_factory);
        let mut op = build_operator_clone(&workload);

        Box::new(move || {
            run_new(&mut op, events);
        })
    })
}

pub fn create_arc_factory<I, F>(
    workload: &Workload,
    event_factory: F,
) -> RunnerFactory
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    F: Fn(Time) -> Event<I> + Clone + 'static,
{
    let workload = workload.clone();

    Box::new(move || {
        let events = create_events_for_workload(&workload, &event_factory);
        let mut op = build_operator_arc(&workload);

        Box::new(move || {
            run_new(&mut op, events);
        })
    })
}

pub fn create_rc_factory<I, F>(
    workload: &Workload,
    event_factory: F,
) -> RunnerFactory
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    F: Fn(Time) -> Event<I> + Clone + 'static,
{
    let workload = workload.clone();

    Box::new(move || {
        let events = create_events_for_workload(&workload, &event_factory);
        let mut op = build_operator_rc(&workload);
        Box::new(move || {
            run_new(&mut op, events);
        })
    })
}