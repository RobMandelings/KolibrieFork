use std::{env, fs};
use criterion::measurement::WallTime;
use criterion::{black_box, BenchmarkGroup, BenchmarkId, Criterion, Throughput};
use dhat::Profiler;
use kolibrie::rsp_engine::bench_pipeline_helper::{preload_city_q1, run_full_pipeline, run_full_pipeline_legacy, CachedGraphs};
use kolibrie::rsp_engine::helpers::init_logger;
use pprof::criterion::{Output, PProfProfiler};
use prototypes::bench_common::{
    copy_group_dir_with_catch, move_profile_file, parse_args, should_run, Strategy,
};
use prototypes::{ArcStrategy, CloneStrategy, SliceStrategy, RcStrategy, WindowSnapshotStrategy};
use shared::triple::Triple;
use std::path::{Path, PathBuf};
use kolibrie::rsp_engine::query_builders::{build_q1_query};
use prototypes::bench_config_parser::resolve_output_config;
use prototypes::workloads::{write_workload_to_file, Workload};

const ROOT: &str = "/Users/robmandelings/Documents/KULeuven/Thesis/KolibrieFork/origin-main";
const DST_ROOT: &str = "../analysis/evaluation";

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn run_mem_profile<S>(cached: &CachedGraphs, pipeline_workload: &Workload)
where
    S: WindowSnapshotStrategy<Triple> + 'static,
{
    let _profiler = Profiler::new_heap();
    run_full_pipeline::<S>(cached, &build_q1_query(&pipeline_workload.window));
}

pub fn run_mem_profile_legacy(cached: &CachedGraphs, pipeline_workload: &Workload)
{
    let _profiler = Profiler::new_heap();
    run_full_pipeline_legacy(black_box(&cached), &build_q1_query(&pipeline_workload.window));
}

pub fn bench_city_q1(
    group: &mut BenchmarkGroup<'_, WallTime>,
    only: &Option<Vec<Strategy>>,
    group_path: &Path,
    workload: &Workload,
    cached_graphs: CachedGraphs
) {
    use Strategy::*;

    init_logger(log::LevelFilter::Info);
    group.throughput(Throughput::Elements(workload.nr_events as u64));

    for strategy in [Slice, Rc, Arc, Clone, Legacy] {
        if !should_run(only, strategy) {
            continue;
        }

        let label = strategy.as_str();

        match strategy {
            Slice => {
                run_bench_and_profile::<SliceStrategy<Triple>>(
                    group,
                    label,
                    group_path,
                    cached_graphs.clone(),
                    workload,
                );
            }
            Rc => {
                run_bench_and_profile::<RcStrategy<Triple>>(
                    group,
                    label,
                    group_path,
                    cached_graphs.clone(),
                    workload,
                );
            }
            Arc => {
                run_bench_and_profile::<ArcStrategy<Triple>>(
                    group,
                    label,
                    group_path,
                    cached_graphs.clone(),
                    workload,
                );
            }
            Clone => {
                run_bench_and_profile::<CloneStrategy<Triple>>(
                    group,
                    label,
                    group_path,
                    cached_graphs.clone(),
                    workload,
                );
            }
            Legacy => {
                group.bench_with_input(
                    BenchmarkId::from_parameter(label),
                    &cached_graphs,
                    |b, cached| {
                        b.iter(|| {
                            run_full_pipeline_legacy(
                                black_box(cached),
                                &build_q1_query(&workload.window)
                            );
                        });
                    },
                );

                run_mem_profile_legacy(black_box(&cached_graphs), &workload);
                move_profile_file(label, group_path);
            }
        }
    }
}

fn run_bench_and_profile<S>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    strategy_name: &str,
    group_path: &Path,
    cached_graphs: CachedGraphs,
    pipeline_workload: &Workload,
) where
    S: WindowSnapshotStrategy<Triple> + 'static,
{
    init_logger(log::LevelFilter::Info);
    group.bench_with_input(
        BenchmarkId::from_parameter(strategy_name),
        &cached_graphs,
        |b, cached| {
            b.iter(|| {
                run_full_pipeline::<S>(black_box(cached), &build_q1_query(&pipeline_workload.window));
            });
        },
    );
    run_mem_profile::<S>(&cached_graphs, pipeline_workload);
    move_profile_file(strategy_name, group_path);
}

fn run_city_q1_once<S>(
    strategy_name: &str,
    cached_graphs: &CachedGraphs,
    pipeline_workload: &Workload,
) where
    S: WindowSnapshotStrategy<Triple> + 'static,
{
    init_logger(log::LevelFilter::Info);

    println!(
        "Running strategy '{}' once for workload '{}'",
        strategy_name, pipeline_workload.name
    );

    run_full_pipeline::<S>(cached_graphs, &build_q1_query(&pipeline_workload.window));
}

fn run_city_q1_once_legacy(
    cached_graphs: &CachedGraphs,
    pipeline_workload: &Workload,
) {
    init_logger(log::LevelFilter::Info);

    println!(
        "Running strategy 'legacy' once for workload '{}'",
        pipeline_workload.name
    );

    run_full_pipeline_legacy(cached_graphs, &build_q1_query(&pipeline_workload.window));
}

fn run_single_city_strategy(
    workload: &Workload,
    strategy: Strategy,
    cached_graphs: CachedGraphs,
    dst_group_path: &Path,
) {
    use Strategy::*;

    fs::create_dir_all(dst_group_path)
        .expect("failed to create workload output directory");

    let label = strategy.as_str();

    match strategy {
        Slice => {
            run_city_q1_once::<SliceStrategy<Triple>>(
                label,
                &cached_graphs,
                workload,
            );
        }
        Rc => {
            run_city_q1_once::<RcStrategy<Triple>>(
                label,
                &cached_graphs,
                workload,
            );
        }
        Arc => {
            run_city_q1_once::<ArcStrategy<Triple>>(
                label,
                &cached_graphs,
                workload,
            );
        }
        Clone => {
            run_city_q1_once::<CloneStrategy<Triple>>(
                label,
                &cached_graphs,
                workload,
            );
        }
        Legacy => {
            run_city_q1_once_legacy(
                &cached_graphs,
                workload,
            );
        }
    }
}

fn main() {
    let cwd = env::current_dir().expect("cannot get working directory");
    println!("cwd: {}", cwd.display());

    let root: PathBuf = cwd
        .parent()
        .map(|p| p.to_path_buf())
        .expect("cwd has no parent");
    println!("root (one up): {}", root.display());

    let args = parse_args();
    let only = args.only;
    let no_bench = args.no_bench;

    let output_cfg = resolve_output_config();
    let dst_root = output_cfg.dst.join(&args.folder_name);
    let criterion_src = output_cfg.criterion_src;
    let dst_root_path = dst_root.join("raw");

    println!(
        "destination path for benchmarks:: {}",
        dst_root_path.display()
    );

    fs::create_dir_all(&dst_root)
        .expect("failed to create benchmark output root");

    let command_file = dst_root.join("command.txt");
    fs::write(&command_file, format!("{}\n", args.raw_command))
        .expect("failed to write command.txt");

    let stream_path = output_cfg.streams.join("AarhusTrafficData158505.stream");
    println!("Stream path: {:?}", &stream_path);
    match no_bench {
        Some(strategy) => {
            for workload in &args.workloads {
                let dst_group_path = dst_root_path.join(&workload.name);

                let cached_graphs = preload_city_q1(workload, &stream_path);
                run_single_city_strategy(workload, strategy, cached_graphs, &dst_group_path);

                let workload_path =
                    format!("{}/workload.json", dst_group_path.to_str().unwrap());
                write_workload_to_file(workload, &workload_path)
                    .expect(&format!("Could not write workload: {}", workload_path));
            }
        }
        None => {
            let sample_size = args
                .sample_size
                .expect("No sample size was parsed; Provide sample size.");

            let mut c: Criterion = Criterion::default()
                .sample_size(sample_size)
                .with_profiler(PProfProfiler::new(
                    100,
                    Output::Flamegraph(None),
                ))
                .with_output_color(true);

            for workload in &args.workloads {
                let mut group = c.benchmark_group(&workload.get_short_name());
                let dst_group_path = dst_root_path.join(&workload.name);
                let criterion_dst_path = dst_group_path.join("throughput");
                let criterion_workload_src = criterion_src.join(&workload.get_short_name());

                let cached_graphs = preload_city_q1(workload, &stream_path);
                bench_city_q1(
                    &mut group,
                    &only,
                    &dst_group_path,
                    workload,
                    cached_graphs
                );

                group.finish();
                copy_group_dir_with_catch(&criterion_workload_src, &criterion_dst_path);

                let workload_path =
                    format!("{}/workload.json", dst_group_path.to_str().unwrap());
                write_workload_to_file(workload, &workload_path)
                    .expect(&format!("Could not write workload: {}", workload_path));

                println!(
                    "Successfully copied criterion group '{}' from '{}' to '{}'",
                    workload.name,
                    criterion_workload_src.display(),
                    criterion_dst_path.display(),
                );
            }

            c.final_summary();
        }
    }
}