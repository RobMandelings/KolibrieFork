// src/lib.rs
pub mod prototype;
pub mod bench_helpers;
pub mod profile_helper;
pub mod experiment_helpers;
pub mod workloads;
pub mod s2r;
pub mod bench_common;
pub mod bench_config_parser;

pub type IRI = String;

pub use prototype::{
    event::{make_string_event, Event},
    slide_strategy::{
        arc_strategy::ArcStrategy,
        clone_strategy::CloneStrategy,
        slice_strategy::SliceStrategy,
        rc_strategy::RcStrategy,
        WindowSnapshotStrategy
    },
    sliding_window_op::SlidingWindowOperator,
    window_params::WindowParams
};

pub use bench_helpers::{
    create_arc_factory,
    create_slice_factory,
    create_rc_factory,
};

pub use profile_helper::run_mem_profile;
pub use experiment_helpers::WINDOW_CONFIGS;