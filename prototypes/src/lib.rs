// src/lib.rs
pub mod prototype;
pub mod bench_helpers;
pub mod profile_helper;
pub mod experiment_helpers;
pub mod workloads;

pub type IRI = String;

pub use prototype::{
    event::Event,
    helpers::event,
    slide_strategy::{
        clone_strategy::CloneStrategy,
        expire_strategy::ExpireStrategy,
        rc_strategy::RcStrategy,
        arc_strategy::ArcStrategy,
        WindowSnapshotStrategy
    },
    sliding_window_op::SlidingWindowOperator,
    window_params::WindowParams
};

pub use bench_helpers::{
    run_strategy_arc,
    run_strategy_expire,
    run_strategy_refcount,
};

pub use profile_helper::run_mem_profile;
pub use experiment_helpers::WINDOW_CONFIGS;