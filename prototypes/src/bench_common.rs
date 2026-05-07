use std::{env, fs, io};
use std::fmt::format;
use std::path::Path;
use log::{debug, info};
use crate::prototype::event::Time;
use crate::workloads::{create_workload, Workload};

pub fn move_profile_file(strat: &str, group_path: &Path) {
    let dir = group_path.join("memory");
    // create mem_profiles/workload_name if needed
    fs::create_dir_all(&dir).expect(&format!("failed to create mem_profiles dir: {:?}", dir));
    let dest = dir.join(format!("{strat}.json"));
    println!("Moving profile file to {:?}", dest);

    fs::rename("dhat-heap.json", &dest).expect("failed to move dhat-heap.json");
}

pub fn copy_dir_recursive(src_group: &Path, dst_group: &Path) -> io::Result<()> {
    fs::create_dir_all(dst_group)?;

    for entry in fs::read_dir(src_group)? {
        let entry = entry?;
        let src_path = entry.path();
        info!("Copying from {}", src_path.to_str().unwrap());

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

pub fn copy_group_dir(src_path: &Path, dst_path: &Path) -> io::Result<()> {
    copy_dir_recursive(&src_path, &dst_path)
}

pub fn copy_group_dir_with_catch(src_path: &Path, dst_path: &Path) -> () {
    match copy_group_dir(&src_path, &dst_path) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("copy failed: {e}");
            eprintln!("src = {}", src_path.display());
            eprintln!("src exists = {}", src_path.exists());
            eprintln!("dst = {}", dst_path.display());
            eprintln!("dst exists = {}", dst_path.exists());
            eprintln!("dst parent = {:?}", dst_path.parent());
            panic!("failed to copy criterion group");
        }
    }
}

pub fn parse_folder_name() -> String {
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--name" {
            return args.next().expect("expected a folder name after --name");
        }
    }

    "".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strategy {
    Clone,
    Slice,
    Rc,
    Legacy,
    Arc,
}

impl Strategy {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "clone" => Some(Self::Clone),
            "expire" => Some(Self::Slice),
            "rc" => Some(Self::Rc),
            "legacy" => Some(Self::Legacy),
            "arc" => Some(Self::Arc),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Clone => "clone",
            Self::Slice => "expire",
            Self::Rc => "rc",
            Self::Legacy => "legacy",
            Self::Arc => "arc",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Args {
    pub folder_name: String,
    pub only: Option<Vec<Strategy>>,
    pub raw_command: String,
    pub workloads: Vec<Workload>,
    pub sample_size: usize,
}

#[derive(Clone, Debug)]
pub(crate) enum WorkloadDim {
    NrWindows(Vec<usize>),
    NrEvents(Vec<usize>),
    EventSpread(Vec<usize>),
    EventOffset(Vec<usize>),
    Bytes(Vec<usize>),
    Size(Vec<usize>),
    Slide(Vec<usize>),
}

impl WorkloadDim {
    pub fn key(&self) -> &'static str {
        match self {
            WorkloadDim::NrWindows(_) => "nr_windows",
            WorkloadDim::NrEvents(_) => "nr_events",
            WorkloadDim::EventSpread(_) => "event_spread",
            WorkloadDim::EventOffset(_) => "event_offset",
            WorkloadDim::Bytes(_) => "bytes",
            WorkloadDim::Size(_) => "size",
            WorkloadDim::Slide(_) => "slide",
        }
    }
}

fn parse_number_list(spec: &str) -> Result<Vec<usize>, String> {
    let mut out = Vec::new();

    for segment in spec.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        if let Some((start, end)) = segment.split_once("..=") {
            let start: usize = start.trim().parse()
                .map_err(|_| format!("invalid range start in '{segment}'"))?;
            let end: usize = end.trim().parse()
                .map_err(|_| format!("invalid range end in '{segment}'"))?;
            if start > end {
                return Err(format!("inclusive range start > end in '{segment}'"));
            }
            out.extend(start..=end);
        } else if let Some((start, end)) = segment.split_once("..") {
            let start: usize = start.trim().parse()
                .map_err(|_| format!("invalid range start in '{segment}'"))?;
            let end: usize = end.trim().parse()
                .map_err(|_| format!("invalid range end in '{segment}'"))?;
            if start > end {
                return Err(format!("exclusive range start > end in '{segment}'"));
            }
            out.extend(start..end);
        } else {
            for part in segment.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                let value = part.parse::<usize>()
                    .map_err(|_| format!("invalid number '{part}' in '{segment}'"))?;
                out.push(value);
            }
        }
    }

    if out.is_empty() {
        return Err(format!("empty numeric spec: '{spec}'"));
    }

    Ok(out)
}

fn parse_usize_list(spec: &str) -> Result<Vec<usize>, String> {
    parse_number_list(spec)
}

fn parse_time_list(spec: &str) -> Result<Vec<Time>, String> {
    parse_number_list(spec)?
        .into_iter()
        .map(|x| {
            x.try_into()
                .map_err(|_| format!("value {x} does not fit into Time"))
        })
        .collect()
}

fn parse_workload_dim(flag: &str, spec: &str) -> Result<WorkloadDim, String> {
    let value = parse_number_list(spec)?;
    match flag {
        "--nr-windows" => Ok(WorkloadDim::NrWindows(value)),
        "--nr-events" => Ok(WorkloadDim::NrEvents(value)),
        "--event-spread" => Ok(WorkloadDim::EventSpread(value)),
        "--event-offset" => Ok(WorkloadDim::EventOffset(value)),
        "--bytes" => Ok(WorkloadDim::Bytes(value)),
        "--size" => Ok(WorkloadDim::Size(value)),
        "--slide" => Ok(WorkloadDim::Slide(value)),
        _ => Err(format!("unknown workload dimension flag: {flag}")),
    }
}

fn build_workloads_from_dims(dims: &[WorkloadDim]) -> Result<Vec<Workload>, String> {
    #[derive(Clone, Debug)]
    struct Partial {
        nr_windows: Option<usize>,
        nr_events: Option<usize>,
        event_spread: Option<usize>,
        event_offset: Option<usize>,
        bytes: Option<usize>,
        size: Option<usize>,
        slide: Option<usize>,
    }

    impl Partial {
        fn new() -> Self {
            Self {
                nr_windows: None,
                nr_events: None,
                event_spread: None,
                event_offset: None,
                bytes: None,
                size: None,
                slide: None,
            }
        }

        fn with_dim(&self, dim: &WorkloadDim, value: usize) -> Self {
            let mut next = self.clone();
            match dim {
                WorkloadDim::NrWindows(_) => next.nr_windows = Some(value),
                WorkloadDim::NrEvents(_) => next.nr_events = Some(value),
                WorkloadDim::EventSpread(_) => next.event_spread = Some(value),
                WorkloadDim::EventOffset(_) => next.event_offset = Some(value),
                WorkloadDim::Bytes(_) => next.bytes = Some(value),
                WorkloadDim::Size(_) => next.size = Some(value),
                WorkloadDim::Slide(_) => next.slide = Some(value),
            }
            next
        }

        fn finalize(self) -> Result<Workload, String> {
            let nr_windows = self.nr_windows.ok_or("missing workload dimension: nr_windows")?;
            let nr_events  = self.nr_events.ok_or("missing workload dimension: nr_events")?;
            let spread_u   = self.event_spread.ok_or("missing workload dimension: event_spread")?;
            let offset_u   = self.event_offset.ok_or("missing workload dimension: event_offset")?;
            let bytes      = self.bytes.ok_or("missing workload dimension: bytes")?;
            let size_u     = self.size.ok_or("missing workload dimension: size")?;
            let slide_u    = self.slide.ok_or("missing workload dimension: slide")?;

            let spread: Time = spread_u as Time;
            let offset: Time = offset_u as Time;
            let size: Time   = size_u as Time;
            let slide: Time  = slide_u as Time;

            Ok(create_workload(
                nr_windows,
                nr_events,
                spread,
                offset,
                bytes,
                size,
                slide,
            ))
        }
    }

    let mut seen = std::collections::HashSet::new();
    for dim in dims {
        let key = dim.key();
        if !seen.insert(key) {
            return Err(format!("duplicate workload dimension provided: {key}"));
        }
    }

    let mut partials = vec![Partial::new()];

    for dim in dims {
        let values: &[usize] = match dim {
            crate::bench_common::WorkloadDim::NrWindows(v) => v,
            crate::bench_common::WorkloadDim::NrEvents(v) => v,
            crate::bench_common::WorkloadDim::EventSpread(v) => v,
            crate::bench_common::WorkloadDim::EventOffset(v) => v,
            crate::bench_common::WorkloadDim::Bytes(v) => v,
            crate::bench_common::WorkloadDim::Size(v) => v,
            crate::bench_common::WorkloadDim::Slide(v) => v,
        };

        let mut next_partials = Vec::with_capacity(partials.len() * values.len());
        for partial in &partials {
            for &value in values {
                next_partials.push(partial.with_dim(dim, value));
            }
        }
        partials = next_partials;
    }

    partials.into_iter().map(|p| p.finalize()).collect()
}

pub fn parse_args() -> Args {

    let raw_args: Vec<String> = env::args().skip(1).collect();
    let raw_command = raw_args.join(" ");
    let all_args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;

    let mut folder_name = String::new();
    let mut only: Vec<Strategy> = Vec::new();
    let mut workload_dims: Vec<WorkloadDim> = Vec::new();
    let mut in_workloads = false;

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
            "--workloads" => {
                in_workloads = true;
            }
            flag @ ("--nr-windows"
            | "--nr-events"
            | "--event-spread"
            | "--event-offset"
            | "--bytes"
            | "--size"
            | "--slide") => {
                if !in_workloads {
                    panic!("{flag} may only be used after --workloads");
                }
                i += 1;
                let spec = all_args
                    .get(i)
                    .unwrap_or_else(|| panic!("expected a value after {flag}"));

                let dim = parse_workload_dim(flag, spec)
                    .unwrap_or_else(|e| panic!("invalid workload spec for {flag}: {e}"));
                workload_dims.push(dim);
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


    if workload_dims.is_empty() {
        panic!("no workloads provided; use --workloads followed by at least one workload dimension");
    }

    let workloads = build_workloads_from_dims(&workload_dims)
        .unwrap_or_else(|e| panic!("could not build workloads: {e}"));

    Args {
        folder_name,
        only: if only.is_empty() { None } else { Some(only) },
        raw_command,
        workloads,
        sample_size: sample_size.expect("No sample size was parsed; Provide sample size."),
    }
}

pub fn should_run(only: &Option<Vec<Strategy>>, strategy: Strategy) -> bool {
    match only {
        None => true,
        Some(list) => list.contains(&strategy),
    }
}