use criterion::measurement::WallTime;
use criterion::{black_box, BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput};
use pprof::criterion::{Output, PProfProfiler};
use pprof::flamegraph::Options;
use pprof::ProfilerGuardBuilder;
use prototypes::bench_common::{copy_group_dir_with_catch, move_profile_file, parse_args, should_run, Strategy};
use prototypes::bench_helpers::{create_clone_factory, create_legacy_factory, RunnerFactory};
use prototypes::prototype::event::{make_byte_event, make_copy_event, Time};
use prototypes::workloads::{default_workloads, write_workload_to_file, Workload};
use prototypes::{run_mem_profile, create_arc_factory, create_expire_factory, create_rc_factory, Event};
use std::fs::File;
use std::path::Path;
use std::{fs, io};
use std::fmt::Debug;
use std::hash::Hash;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const PROFILING_FREQUENCY_HZ: i32 = 100;
const PROFILE_ITERS: usize = 200;
const BLOCKLIST: &[&str] = &["libc", "libgcc", "pthread", "vdso"];

const ROOT: &str = "/Users/robmandelings/Documents/KULeuven/Thesis/KolibrieFork/origin-main";
const DST_ROOT: &str = "../analysis/evaluation";

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

fn run_bench_and_profile(
    group: &mut BenchmarkGroup<'_, WallTime>,
    workload: &Workload,
    label: &str, // "clone"
    group_path: &Path,
    runner_factory: RunnerFactory,
)
{
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

    for strategy in [Clone, Rc, Arc, Legacy, Expire] {
        if !should_run(only, strategy) {
            continue;
        }

        let label = match strategy {
            Clone  => "clone",
            Rc     => "rc",
            Arc    => "arc",
            Legacy => "legacy",
            Expire => "expire",
        };

        let factory = match strategy {
            Clone  => create_clone_factory(workload, make_event),
            Rc     => create_rc_factory(workload, make_event),
            Arc    => create_arc_factory(workload, make_event),
            Legacy => create_legacy_factory(workload, make_event),
            Expire => create_expire_factory(workload, make_event),
        };

        run_bench_and_profile(group, workload, label, dst_group_path, factory);
    }
}

fn main() {
    let args = parse_args();
    let only = args.only;
    let root_path = Path::new(ROOT);
    let prototypes_root_path = root_path.join("prototypes");
    let dst_root_path = prototypes_root_path.join(DST_ROOT).join(&args.folder_name);

    let mut c: Criterion = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(1))
        .with_profiler(PProfProfiler::new(
            100, // sampling frequency (Hz)
            Output::Flamegraph(None),
        ))
        .with_output_color(true);

    let grouped_workloads = default_workloads();
    for (group_name, workloads) in &grouped_workloads {
        let grouped_dst_root_path = dst_root_path.join(group_name);

        for workload in workloads {
            // One group per workload

            // use SHORT name for the criterion benchmark thing because for some reason it has a max filename length
            let mut group = c.benchmark_group(&workload.get_short_name());
            let dst_group_path = grouped_dst_root_path.join(&workload.name);
            let criterion_dst_path = dst_group_path.join("throughput");
            let criterion_src_path = root_path.join("target/criterion").join(&workload.get_short_name()); // Where to take the data from

            match workload.bytes {
                0 => run_benches(&mut group, workload, &only, &dst_group_path, make_copy_event),
                bytes => run_benches(&mut group, workload, &only, &dst_group_path, move |ts| make_byte_event(ts, bytes)),
            }

            group.finish();

            copy_group_dir_with_catch(&criterion_src_path, &criterion_dst_path);

            let workload_path = format!("{}/workload.json", dst_group_path.to_str().unwrap());
            write_workload_to_file(workload, &workload_path)
                .expect(&format!("Could not write workload: {}", workload_path));
            println!(
                "Successfully copied criterion group {} in {}",
                workload.name, group_name
            );
        }
    }

    c.final_summary();
}
