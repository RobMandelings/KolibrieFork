use criterion::measurement::WallTime;
use criterion::{black_box, BenchmarkGroup, BenchmarkId, Criterion, Throughput};
use pprof::criterion::{Output, PProfProfiler};
use pprof::flamegraph::Options;
use pprof::ProfilerGuardBuilder;
use prototypes::bench_common::{copy_group_dir_with_catch, move_profile_file, parse_args, should_run, Strategy};
use prototypes::bench_helpers::{run_strategy_clone, run_strategy_legacy};
use prototypes::prototype::event::{make_byte_event, make_copy_event, Time};
use prototypes::workloads::{default_workloads, write_workload_to_file, Workload};
use prototypes::{run_mem_profile, run_strategy_arc, run_strategy_expire, run_strategy_rc, Event};
use std::fs::File;
use std::path::Path;
use std::{fs, io};

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

pub fn bench_strategy<F>(
    group: &mut BenchmarkGroup<WallTime>,
    strategy_str: &str,
    workload: &Workload,
    mut run_strategy: F,
) where
    F: FnMut(&Workload),
{
    group.throughput(Throughput::Elements(workload.nr_events as u64));

    group.bench_with_input(
        BenchmarkId::from_parameter(strategy_str),
        &workload,
        |b, &workload| {
            b.iter(|| {
                run_strategy(&workload);
            })
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

fn run_bench_and_profile<I, F>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    workload: &Workload,
    label: &str, // "clone"
    group_path: &Path,
    event_factory: F,
    run_strategy: fn(&Workload, Vec<Event<I>>),
)
where
    I: Clone,
    F: Fn(Time) -> Event<I>
{
    let events: Vec<Event<I>> = (0..workload.nr_events as Time)
        .map(event_factory)
        .collect();

    bench_strategy(group, label, workload, |w| run_strategy(w, events.clone()));
    run_mem_profile(label, |w| run_strategy(w, events), workload);
    move_profile_file(label, group_path);
}

fn run_copy_benches(
    group: &mut BenchmarkGroup<WallTime>,
    workload: &Workload,
    only: &Option<Vec<Strategy>>,
    dst_group_path: &Path,
) {
    if should_run(only, Strategy::Clone) {
        run_bench_and_profile(group, workload, "clone", dst_group_path, make_copy_event, run_strategy_clone);
    }
    if should_run(only, Strategy::Rc) {
        run_bench_and_profile(group, workload, "rc", dst_group_path, make_copy_event, run_strategy_rc);
    }
    if should_run(only, Strategy::Arc) {
        run_bench_and_profile(group, workload, "arc", dst_group_path, make_copy_event, run_strategy_arc);
    }
    if should_run(only, Strategy::Legacy) {
        run_bench_and_profile(group, workload, "legacy", dst_group_path, make_copy_event, run_strategy_legacy);
    }
    if should_run(only, Strategy::Expire) {
        run_bench_and_profile(group, workload, "expire", dst_group_path, make_copy_event, run_strategy_expire);
    }
}

fn run_byte_benches(
    group: &mut BenchmarkGroup<WallTime>,
    workload: &Workload,
    only: &Option<Vec<Strategy>>,
    dst_group_path: &Path,
    bytes: usize,
) {
    if should_run(only, Strategy::Clone) {
        run_bench_and_profile(group, workload, "clone", dst_group_path, |ts| make_byte_event(ts, bytes), run_strategy_clone);
    }
    if should_run(only, Strategy::Rc) {
        run_bench_and_profile(group, workload, "rc", dst_group_path, |ts| make_byte_event(ts, bytes), run_strategy_rc);
    }
    if should_run(only, Strategy::Arc) {
        run_bench_and_profile(group, workload, "arc", dst_group_path, |ts| make_byte_event(ts, bytes), run_strategy_arc);
    }
    if should_run(only, Strategy::Legacy) {
        run_bench_and_profile(group, workload, "legacy", dst_group_path, |ts| make_byte_event(ts, bytes), run_strategy_legacy);
    }
    if should_run(only, Strategy::Expire) {
        run_bench_and_profile(group, workload, "expire", dst_group_path, |ts| make_byte_event(ts, bytes), run_strategy_expire);
    }
}

fn main() {
    let args = parse_args();
    let only = args.only;
    let root_path = Path::new(ROOT);
    let prototypes_root_path = root_path.join("prototypes");
    let dst_root_path = prototypes_root_path.join(DST_ROOT).join(&args.folder_name);

    let mut c: Criterion = Criterion::default()
        .with_profiler(PProfProfiler::new(
            100, // sampling frequency (Hz)
            Output::Flamegraph(None),
        ))
        .with_output_color(true);
    let workloads = default_workloads();

    for workload in &workloads {
        // One group per workload
        let mut group = c.benchmark_group(&workload.name);
        let dst_group_path = dst_root_path.join(&workload.name);
        let criterion_dst_path = dst_group_path.join("throughput");
        let criterion_src_path = root_path.join("target/criterion").join(&workload.name); // Where to take the data from

        match workload.bytes {
            0 => run_copy_benches(&mut group, workload, &only, &dst_group_path),
            bytes => run_byte_benches(&mut group, workload, &only, &dst_group_path, bytes),
        }

        group.finish();

        copy_group_dir_with_catch(&criterion_src_path, &criterion_dst_path);

        let workload_path = format!("{}/workload.json", dst_group_path.to_str().unwrap());
        write_workload_to_file(workload, &workload_path)
            .expect(&format!("Could not write workload: {}", workload_path));
        println!("Successfully copied criterion group {}", workload.name);
    }

    c.final_summary();
}
