use std::clone::Clone;
use criterion::measurement::WallTime;
use criterion::{black_box, BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput};
use pprof::criterion::{Output, PProfProfiler};
use pprof::flamegraph::Options;
use pprof::ProfilerGuardBuilder;
use prototypes::bench_common::{copy_group_dir_with_catch, move_profile_file, parse_args, should_run, Strategy};
use prototypes::bench_helpers::{create_clone_factory, create_legacy_factory, RunnerFactory};
use prototypes::prototype::event::{make_byte_event, make_copy_event, Time};
use prototypes::workloads::{write_workload_to_file, Workload};
use prototypes::{run_mem_profile, create_arc_factory, create_slice_factory, create_rc_factory, Event};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use std::fmt::Debug;
use std::hash::Hash;
use prototypes::bench_config_parser::{resolve_output_config};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const PROFILING_FREQUENCY_HZ: i32 = 100;
const PROFILE_ITERS: usize = 200;
const BLOCKLIST: &[&str] = &["libc", "libgcc", "pthread", "vdso"];

fn ensure_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

pub fn bench_strategy(
    group: &mut BenchmarkGroup<WallTime>,
    strategy_str: &str,
    nr_events: usize,
    setup_runner: &RunnerFactory,
)
{
    group.throughput(Throughput::Elements(nr_events as u64));
    group.bench_function(
        BenchmarkId::from_parameter(strategy_str),
        |b| {
            b.iter_batched(
                || {
                    setup_runner()
                },
                |runner| {
                    runner();
                },
                BatchSize::SmallInput,
            );
        },
    );
}

fn write_flamegraph_for_strategy<F>(
    workload: &Workload,
    strategy: &str,
    group_path: &Path,
    run_strategy: F,
) where
    F: Fn(&Workload) + Copy,
{
    let out_dir = group_path.join("flamegraph");
    ensure_dir(&out_dir).expect("failed to create output directory");

    let flamegraph_path = out_dir.join(format!("{strategy}.svg"));

    let guard = ProfilerGuardBuilder::default()
        .frequency(PROFILING_FREQUENCY_HZ)
        .blocklist(BLOCKLIST)
        .build()
        .expect("failed to start profiler");

    for _ in 0..PROFILE_ITERS {
        black_box(run_strategy(black_box(workload)));
    }

    let report = guard
        .report()
        .build()
        .expect("failed to build pprof report");

    let file = File::create(&flamegraph_path).expect("failed to create flamegraph.svg");

    let mut options = Options::default();
    options.image_width = Some(2400);

    report
        .flamegraph_with_options(file, &mut options)
        .expect("failed to write flamegraph");
}

fn print_running_benchmark(strategy: &str, workload: &Workload) {
    println!(
        "[BENCH] strategy={strategy} | windows={} | events={} | size={} | slide={} | spread={} | event_offset={} | bytes={} | reserve={}",
        workload.nr_windows,
        workload.nr_events,
        workload.window.size,
        workload.window.slide,
        workload.stream_config.spread,
        workload.stream_config.offset,
        workload.bytes,
        workload.reserve
    );
}

fn make_factory<I, E>(
    strategy: Strategy,
    workload: &Workload,
    make_event: E,
) -> RunnerFactory
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    E: Fn(Time) -> Event<I> + Copy + 'static,
{
    use Strategy::*;
    match strategy {
        Slice => create_slice_factory(workload, make_event),
        Rc => create_rc_factory(workload, make_event),
        Arc => create_arc_factory(workload, make_event),
        Clone => create_clone_factory(workload, make_event),
        Legacy => create_legacy_factory(workload, make_event),
    }
}

fn run_bench_and_profile(
    group: &mut BenchmarkGroup<'_, WallTime>,
    workload: &Workload,
    label: &str, // "clone"
    group_path: &Path,
    runner_factory: RunnerFactory,
)
{
    print_running_benchmark(label, workload);
    bench_strategy(group, label, workload.nr_events, &runner_factory);
    run_mem_profile(label, &runner_factory);
    move_profile_file(label, group_path);
}

fn run_benches<I, E>(
    group: &mut BenchmarkGroup<WallTime>,
    workload: &Workload,
    only: &Option<Vec<Strategy>>,
    dst_group_path: &Path,
    make_event: E,
) where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    E: Fn(Time) -> Event<I> + Copy + 'static,
{
    use Strategy::*;

    for strategy in [Slice, Rc, Arc, Clone, Legacy] {
        if !should_run(only, strategy) {
            continue;
        }

        let label = strategy.as_str();
        let factory = make_factory(strategy, workload, make_event);
        run_bench_and_profile(group, workload, label, dst_group_path, factory);
    }
}

fn run_single_strategy<I, E>(
    workload: &Workload,
    strategy: Strategy,
    dst_group_path: &Path,
    make_event: E,
)
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    E: Fn(Time) -> Event<I> + Copy + 'static,
{

    fs::create_dir_all(dst_group_path)
        .expect("failed to create workload output directory");

    let label = strategy.as_str();
    println!(
        "Running strategy '{}' once for workload '{}'",
        label, workload.name
    );

    let factory = make_factory(strategy, workload, make_event);
    let runner = factory();
    runner();
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
    println!("destination path for benchmarks:: {}", dst_root_path.display());

    fs::create_dir_all(&dst_root)
        .expect("failed to create benchmark output root");

    let command_file = dst_root.join("command.txt");
    fs::write(&command_file, format!("{}\n", args.raw_command))
        .expect("failed to write command.txt");

    match no_bench {
        Some(strategy) => {
            for workload in &args.workloads {
                let dst_group_path = dst_root_path.join(&workload.name);

                match workload.bytes {
                    0 => run_single_strategy(workload, strategy, &dst_group_path, make_copy_event),
                    bytes => run_single_strategy(workload, strategy, &dst_group_path, move |ts| make_byte_event(ts, bytes)),
                }

                let workload_path = format!("{}/workload.json", dst_group_path.to_str().unwrap());
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

                match workload.bytes {
                    0 => run_benches(&mut group, workload, &only, &dst_group_path, make_copy_event),
                    bytes => run_benches(&mut group, workload, &only, &dst_group_path, move |ts| make_byte_event(ts, bytes)),
                }

                group.finish();
                copy_group_dir_with_catch(&criterion_workload_src, &criterion_dst_path);

                let workload_path = format!("{}/workload.json", dst_group_path.to_str().unwrap());
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