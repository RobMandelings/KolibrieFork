use std::env;
use std::fmt::Debug;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::{fs, io};

use criterion::measurement::WallTime;
use criterion::{
    black_box, BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput,
};
use prototypes::bench_common::Strategy;
use prototypes::Event;
use prototypes::prototype::event::{make_copy_event, Time, Triple};
use prototypes::prototype::slide_strategy::strategies_report_only::{create_arc_report, create_clone_report, create_rc_report, create_slice_report};

#[derive(Clone, Debug)]
pub struct Args {
    pub folder_name: String,
    pub only: Option<Vec<Strategy>>,
    pub sizes: Vec<usize>,
    pub sample_size: usize,
    pub raw_command: String,
}

pub fn parse_args() -> Args {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let raw_command = raw_args.join(" ");
    let all_args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;

    let mut folder_name = String::new();
    let mut only: Vec<Strategy> = Vec::new();
    let mut sizes: Vec<usize> = Vec::new();
    let mut sample_size: Option<usize> = None;

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
            "--size" => {
                i += 1;
                while i < all_args.len() && !all_args[i].starts_with("--") {
                    let parsed = all_args[i]
                        .parse::<usize>()
                        .unwrap_or_else(|_| panic!("invalid usize for --size: {}", all_args[i]));
                    sizes.push(parsed);
                    i += 1;
                }
                continue;
            }
            "--sample-size" => {
                i += 1;
                let value = all_args
                    .get(i)
                    .unwrap_or_else(|| panic!("expected a number after --sample-size"));
                let parsed: usize = value
                    .parse()
                    .unwrap_or_else(|_| panic!("invalid usize for --sample-size: {value}"));

                if parsed < 10 {
                    panic!("--sample-size must be at least 10");
                }

                sample_size = Some(parsed);
            }
            other => {
                panic!("unknown argument: {other}");
            }
        }
        i += 1;
    }

    if folder_name.is_empty() {
        panic!("missing required argument: --name <folder>");
    }

    if sizes.is_empty() {
        panic!("missing required argument: --size <n1> <n2> ...");
    }

    Args {
        folder_name,
        only: if only.is_empty() { None } else { Some(only) },
        sizes,
        sample_size: sample_size.expect("No sample size was parsed; Provide sample size."),
        raw_command,
    }
}

fn ensure_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

fn should_run(only: &Option<Vec<Strategy>>, strategy: Strategy) -> bool {
    match only {
        None => true,
        Some(v) => v.contains(&strategy),
    }
}

fn make_content(size: usize) -> Vec<Event<Triple>> {
    (0..size)
        .map(|ts| make_copy_event(ts as Time))
        .collect()
}

fn make_rc_content(size: usize) -> Vec<Rc<Event<Triple>>> {
    make_content(size).into_iter().map(Rc::new).collect()
}

fn make_arc_content(size: usize) -> Vec<Arc<Event<Triple>>> {
    make_content(size).into_iter().map(Arc::new).collect()
}

fn bench_slice_report(group: &mut BenchmarkGroup<WallTime>, size: usize) {
    group.bench_with_input(BenchmarkId::new("slice", size), &size, |b, &size| {
        b.iter_batched(
            || make_content(size),
            |content| {
                black_box(create_slice_report(&content));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_clone_report(group: &mut BenchmarkGroup<WallTime>, size: usize) {
    group.bench_with_input(BenchmarkId::new("clone", size), &size, |b, &size| {
        b.iter_batched(
            || make_content(size),
            |content| {
                black_box(create_clone_report(&content, 0));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_rc_report(group: &mut BenchmarkGroup<WallTime>, size: usize) {
    group.bench_with_input(BenchmarkId::new("rc", size), &size, |b, &size| {
        b.iter_batched(
            || make_rc_content(size),
            |content| {
                black_box(create_rc_report(&content));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_arc_report(group: &mut BenchmarkGroup<WallTime>, size: usize) {
    group.bench_with_input(BenchmarkId::new("arc", size), &size, |b, &size| {
        b.iter_batched(
            || make_arc_content(size),
            |content| {
                black_box(create_arc_report(&content));
            },
            BatchSize::SmallInput,
        );
    });
}

fn run_report_benches(
    group: &mut BenchmarkGroup<WallTime>,
    size: usize,
    only: &Option<Vec<Strategy>>,
) {
    use Strategy::*;

    if should_run(only, Slice) {
        bench_slice_report(group, size);
    }
    if should_run(only, Clone) {
        bench_clone_report(group, size);
    }
    if should_run(only, Rc) {
        bench_rc_report(group, size);
    }
    if should_run(only, Arc) {
        bench_arc_report(group, size);
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
    let only = args.only.clone();

    let dst_root = root.join("benchmark-results").join(&args.folder_name);
    ensure_dir(&dst_root).expect("failed to create benchmark output root");

    let command_file = dst_root.join("command.txt");
    fs::write(&command_file, format!("{}\n", args.raw_command))
        .expect("failed to write command.txt");

    let mut c: Criterion = Criterion::default()
        .sample_size(args.sample_size)
        .with_output_color(true);

    let mut group = c.benchmark_group("create_report");
    for &size in &args.sizes {
        group.throughput(Throughput::Elements(size as u64));
        run_report_benches(&mut group, size, &only);
    }
    group.finish();

    c.final_summary();
}