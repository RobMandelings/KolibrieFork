use criterion::measurement::WallTime;
use criterion::{black_box, BenchmarkGroup, BenchmarkId, Criterion, Throughput};
use dhat::Profiler;
use kolibrie::rsp_engine::bench_pipeline_helper::{preload_city_q1_two_window_graphs, run_full_pipeline_bench, run_legacy_pipeline_bench, CachedGraphs};
use kolibrie::rsp_engine::helpers::init_logger;
use kolibrie::rsp_engine::pipeline_workload::{default_workloads, write_workload_to_file, PipelineWorkload};
use pprof::criterion::{Output, PProfProfiler};
use prototypes::bench_common::{
    copy_group_dir_with_catch, move_profile_file, parse_args, should_run, Strategy,
};
use prototypes::{ArcStrategy, CloneStrategy, SliceStrategy, RcStrategy, WindowSnapshotStrategy};
use shared::triple::Triple;
use std::path::Path;
use kolibrie::rsp_engine::query_builders::{build_q1_query};

const ROOT: &str = "/Users/robmandelings/Documents/KULeuven/Thesis/KolibrieFork/origin-main";
const DST_ROOT: &str = "../analysis/evaluation";

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn run_mem_profile<S>(cached: &CachedGraphs, pipeline_workload: &PipelineWorkload)
where
    S: WindowSnapshotStrategy<Triple> + 'static,
{
    let _profiler = Profiler::new_heap();
    run_full_pipeline_bench::<S>(cached, &build_q1_query(&pipeline_workload.window_params));
}

pub fn run_mem_profile_legacy(cached: &CachedGraphs, pipeline_workload: &PipelineWorkload)
{
    let _profiler = Profiler::new_heap();
    run_legacy_pipeline_bench(black_box(&cached), &build_q1_query(&pipeline_workload.window_params));
}

pub fn bench_city_q1_two_windows(
    group: &mut BenchmarkGroup<'_, WallTime>,
    only: &Option<Vec<Strategy>>,
    group_path: &Path,
    pipeline_workload: &PipelineWorkload
) {
    init_logger(log::LevelFilter::Info);
    group.throughput(Throughput::Elements(pipeline_workload.nr_events as u64));
    let cached_graphs = preload_city_q1_two_window_graphs(pipeline_workload.nr_events);
    if should_run(only, Strategy::Clone) {
        run_bench_and_profile::<CloneStrategy<Triple>>(
            group,
            "clone",
            group_path,
            cached_graphs.clone(),
            pipeline_workload,
        );
    }

    if should_run(only, Strategy::Rc) {
        run_bench_and_profile::<RcStrategy<Triple>>(
            group,
            "rc",
            group_path,
            cached_graphs.clone(),
            pipeline_workload,
        );
    }

    if should_run(only, Strategy::Arc) {
        run_bench_and_profile::<ArcStrategy<Triple>>(
            group,
            "arc",
            group_path,
            cached_graphs.clone(),
            pipeline_workload,
        );
    }

    if should_run(only, Strategy::Slice) {
        run_bench_and_profile::<SliceStrategy<Triple>>(
            group,
            "expire",
            group_path,
            cached_graphs.clone(),
            pipeline_workload,
        );
    }

    if should_run(only, Strategy::Legacy) {
        group.bench_with_input(
            BenchmarkId::from_parameter("legacy"),
            &cached_graphs,
            |b, cached| {
                b.iter(|| {
                    run_legacy_pipeline_bench(black_box(cached), &build_q1_query(&pipeline_workload.window_params));
                });
            },
        );

        run_mem_profile_legacy(black_box(&cached_graphs), &pipeline_workload);
        move_profile_file("legacy", group_path);
    }
}

fn run_bench_and_profile<S>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    strategy_name: &str,
    group_path: &Path,
    cached_graphs: CachedGraphs,
    pipeline_workload: &PipelineWorkload,
) where
    S: WindowSnapshotStrategy<Triple> + 'static,
{
    init_logger(log::LevelFilter::Info);
    group.bench_with_input(
        BenchmarkId::from_parameter(strategy_name),
        &cached_graphs,
        |b, cached| {
            b.iter(|| {
                run_full_pipeline_bench::<S>(black_box(cached), &build_q1_query(&pipeline_workload.window_params));
            });
        },
    );
    run_mem_profile::<S>(&cached_graphs, pipeline_workload);
    move_profile_file(strategy_name, group_path);
}

fn main() {
    // Parse CLI flags as in your original code
    let args = parse_args();
    println!("Only: {:?}", args.only);
    let root_path = Path::new(ROOT);
    let prototypes_root_path = root_path.join("prototypes");
    let dst_root_path = prototypes_root_path.join(DST_ROOT).join(&args.folder_name);
    let workloads = default_workloads();

    // Set up Criterion with pprof profiler
    let mut c: Criterion = Criterion::default()
        .with_profiler(PProfProfiler::new(
            100, // sampling frequency (Hz)
            Output::Flamegraph(None),
        ))
        .with_output_color(true);

    for workload in &workloads {
        // One group per workload
        let mut group = c.benchmark_group(&workload.name);
        let dst_group_path = dst_root_path.join(&workload.name);
        let criterion_dst_path = dst_group_path.join("throughput");
        let criterion_src_path = root_path.join("target/criterion").join(&workload.name); // Where to take the data from

        bench_city_q1_two_windows(&mut group, &args.only, &dst_group_path, workload);
        group.finish();

        copy_group_dir_with_catch(&criterion_src_path, &criterion_dst_path);

        let workload_path = format!("{}/workload.json", dst_group_path.to_str().unwrap());
        write_workload_to_file(workload, &workload_path)
            .expect(&format!("Could not write workload: {}", workload_path));

        println!("Successfully copied criterion group {}", workload.name);
    }

    c.final_summary();
}