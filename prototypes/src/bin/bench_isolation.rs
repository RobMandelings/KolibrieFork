use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, BenchmarkId, Criterion, Throughput, black_box};
use pprof::ProfilerGuardBuilder;
use pprof::criterion::{Output, PProfProfiler};
use pprof::flamegraph::Options;
use prototypes::bench_helpers::{run_strategy_clone, run_strategy_legacy, EventFactory};
use prototypes::workloads::{Workload, default_workloads, write_workload_to_file};
use prototypes::{run_mem_profile, run_strategy_arc, run_strategy_expire, run_strategy_rc, Event};
use std::fs::File;
use std::path::Path;
use std::{env, fs, io};
use prototypes::prototype::event::{make_string_event, Time};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const PROFILING_FREQUENCY_HZ: i32 = 100;
const PROFILE_ITERS: usize = 200;
const BLOCKLIST: &[&str] = &["libc", "libgcc", "pthread", "vdso"];

const ROOT: &str = "/Users/robmandelings/Documents/KULeuven/Thesis/KolibrieFork/origin-main";
const DST_ROOT: &str = "analysis/evaluation";

fn ensure_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

pub fn move_profile_file(strat: &str, group_path: &Path) {
    let dir = group_path.join("memory");
    // create mem_profiles/workload_name if needed
    fs::create_dir_all(&dir).expect("failed to create mem_profiles dir");
    let dest = dir.join(format!("{strat}.json"));

    fs::rename("dhat-heap.json", &dest).expect("failed to move dhat-heap.json");
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

fn run_bench_and_profile<I>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    workload: &Workload,
    label: &str, // "clone"
    group_path: &Path,
    event_factory: EventFactory<I>,
    run_strategy: fn(&Workload, Vec<Event<I>>),
)
where I: Clone
{
    let events: Vec<Event<I>> = (0..workload.nr_events as Time)
        .map(event_factory)
        .collect();

    bench_strategy(group, label, workload, |w| run_strategy(w, events.clone()));

    run_mem_profile(label, |w| run_strategy(w, events), workload);
    move_profile_file(label, group_path);

    // write_flamegraph_for_strategy(workload, label, group_path, run_strategy);
}

fn copy_dir_recursive(src_group: &Path, dst_group: &Path) -> io::Result<()> {
    fs::create_dir_all(dst_group)?;

    for entry in fs::read_dir(src_group)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_type = entry.file_type()?;

        if !file_type.is_dir() {
            continue;
        }

        let strat_name = entry.file_name();
        if strat_name == "report" {
            continue;
        }

        // e.g. target/criterion/<group>/clone/new
        let new_dir = src_path.join("new");
        if !new_dir.is_dir() {
            continue;
        }

        // e.g. analysis/evaluation/<group>/clone
        let dst_strat_dir = dst_group.join(&strat_name);
        fs::create_dir_all(&dst_strat_dir)?;

        for file in fs::read_dir(&new_dir)? {
            let file = file?;
            let src_file = file.path();
            if file.file_type()?.is_file() {
                let dst_file = dst_strat_dir.join(file.file_name());
                fs::rename(&src_file, &dst_file)?;
            }
        }
    }

    Ok(())
}

fn copy_group_dir(src_path: &Path, dst_path: &Path) -> io::Result<()> {
    copy_dir_recursive(&src_path, &dst_path)
}

fn parse_folder_name() -> String {
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--name" {
            return args.next().expect("expected a folder name after --name");
        }
    }

    "".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Strategy {
    Clone,
    Expire,
    Rc,
    Legacy,
    Arc,
}

impl Strategy {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "clone" => Some(Self::Clone),
            "expire" => Some(Self::Expire),
            "rc" => Some(Self::Rc),
            "legacy" => Some(Self::Legacy),
            "arc" => Some(Self::Arc),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Clone => "clone",
            Self::Expire => "expire",
            Self::Rc => "rc",
            Self::Legacy => "legacy",
            Self::Arc => "arc",
        }
    }
}

struct Args {
    folder_name: String,
    only: Option<Vec<Strategy>>,
}

fn parse_args() -> Args {
    let all_args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;

    let mut folder_name = String::new();
    let mut only: Vec<Strategy> = Vec::new();

    while i < all_args.len() {
        match all_args[i].as_str() {
            "--name" => {
                i += 1;
                folder_name = all_args
                    .get(i)
                    .cloned()
                    .expect("expected a folder name after --name");
            }
            "--only" => {
                i += 1;
                while i < all_args.len() && !all_args[i].starts_with("--") {
                    let strategy = Strategy::parse(&all_args[i])
                        .unwrap_or_else(|| panic!("unknown strategy for --only: {}", all_args[i]));
                    only.push(strategy);
                    i += 1;
                }
                continue;
            }
            other => {
                panic!("unknown argument: {other}");
            }
        }
        i += 1;
    }

    Args {
        folder_name,
        only: if only.is_empty() { None } else { Some(only) },
    }
}

fn should_run(only: &Option<Vec<Strategy>>, strategy: Strategy) -> bool {
    match only {
        None => true,
        Some(list) => list.contains(&strategy),
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

        if should_run(&only, Strategy::Clone) {
            run_bench_and_profile(&mut group, workload, "clone", &dst_group_path, make_string_event, run_strategy_clone);
        }

        if should_run(&only, Strategy::Rc) {
            run_bench_and_profile(&mut group, workload, "rc", &dst_group_path, make_string_event, run_strategy_rc);
        }

        if should_run(&only, Strategy::Arc) {
            run_bench_and_profile(&mut group, workload, "arc", &dst_group_path, make_string_event, run_strategy_arc);
        }

        if should_run(&only, Strategy::Legacy) {
            run_bench_and_profile(&mut group, workload, "legacy", &dst_group_path, make_string_event, run_strategy_legacy);
        }

        if should_run(&only, Strategy::Expire) {
            run_bench_and_profile(&mut group, workload, "expire", &dst_group_path, make_string_event, run_strategy_expire);
        }

        group.finish();

        match copy_group_dir(&criterion_src_path, &criterion_dst_path) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("copy failed: {e}");
                eprintln!("src = {}", criterion_src_path.display());
                eprintln!("src exists = {}", criterion_src_path.exists());
                eprintln!("dst = {}", criterion_dst_path.display());
                eprintln!("dst exists = {}", criterion_dst_path.exists());
                eprintln!("dst parent = {:?}", criterion_dst_path.parent());
                panic!("failed to copy criterion group");
            }
        }

        let workload_path = format!("{}/workload.json", dst_group_path.to_str().unwrap());
        write_workload_to_file(workload, &workload_path)
            .expect(&format!("Could not write workload: {}", workload_path));
        println!("Successfully copied criterion group {}", workload.name);
    }

    c.final_summary();
}
