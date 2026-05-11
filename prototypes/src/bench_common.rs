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
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "clone" => Some(Self::Clone),
            "slice" => Some(Self::Slice),
            "rc" => Some(Self::Rc),
            "legacy" => Some(Self::Legacy),
            "arc" => Some(Self::Arc),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clone => "clone",
            Self::Slice => "slice",
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
    pub no_bench: Option<Strategy>,
    pub raw_command: String,
    pub workloads: Vec<Workload>,
    pub sample_size: Option<usize>,
}

#[derive(Clone, Debug)]
pub(crate) enum EventSpreadSpec {
    Values(Vec<usize>),
    FollowSlide,
}

#[derive(Clone, Debug)]
pub(crate) enum NrEventsSpec {
    Values(Vec<usize>),
    Expr(String),
}

#[derive(Clone, Debug)]
pub(crate) enum WorkloadDim {
    NrWindows(Vec<usize>),
    NrEvents(NrEventsSpec),
    EventSpread(EventSpreadSpec),
    EventOffset(Vec<usize>),
    Bytes(Vec<usize>),
    Size(Vec<usize>),
    Slide(Vec<usize>),
    Reserve(Vec<usize>),
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
            WorkloadDim::Reserve(_) => "reserve",
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
    match flag {
        "--nr-windows" => Ok(WorkloadDim::NrWindows(parse_number_list(spec)?)),
        "--nr-events" => {
            let trimmed = spec.trim();
            // digits/commas → list; otherwise treat as expression
            if trimmed.chars().all(|c| {
                c.is_ascii_digit()
                    || c.is_whitespace()
                    || matches!(c, ',' | ';' | '.' | '=')
            }) {
                Ok(WorkloadDim::NrEvents(NrEventsSpec::Values(
                    parse_number_list(trimmed)?,
                )))
            } else {
                Ok(WorkloadDim::NrEvents(NrEventsSpec::Expr(
                    trimmed.to_string(),
                )))
            }
        }
        "--event-spread" => {
            if spec.trim() == "size" {
                panic!("Not implemented yet");
            }
            if spec.trim() == "slide" {
                Ok(WorkloadDim::EventSpread(EventSpreadSpec::FollowSlide))
            } else {
                Ok(WorkloadDim::EventSpread(EventSpreadSpec::Values(parse_number_list(spec)?)))
            }
        }
        "--event-offset" => Ok(WorkloadDim::EventOffset(parse_number_list(spec)?)),
        "--bytes" => Ok(WorkloadDim::Bytes(parse_number_list(spec)?)),
        "--size" => Ok(WorkloadDim::Size(parse_number_list(spec)?)),
        "--slide" => Ok(WorkloadDim::Slide(parse_number_list(spec)?)),
        "--reserve" => Ok(WorkloadDim::Reserve(parse_number_list(spec)?)),
        _ => Err(format!("unknown workload dimension flag: {flag}")),
    }
}

#[derive(Clone, Debug)]
struct Partial {
    nr_windows: Option<usize>,
    nr_events: Option<usize>,
    event_spread: Option<usize>,
    event_spread_follows_slide: bool,
    event_offset: Option<usize>,
    bytes: Option<usize>,
    size: Option<usize>,
    slide: Option<usize>,
    reserve: Option<usize>,
}

impl Partial {
    fn new() -> Self {
        Self {
            nr_windows: None,
            nr_events: None,
            event_spread: None,
            event_spread_follows_slide: false,
            event_offset: None,
            bytes: None,
            size: None,
            slide: None,
            reserve: Some(0),
        }
    }

    fn with_event_spread_follow_slide(&self) -> Self {
        let mut next = self.clone();
        next.event_spread_follows_slide = true;
        next
    }

    fn with_nr_events(&self, value: usize) -> Self {
        let mut next = self.clone();
        next.nr_events = Some(value);
        next
    }

    fn with_dim(&self, dim: &WorkloadDim, value: usize) -> Self {
        let mut next = self.clone();
        match dim {
            WorkloadDim::NrWindows(_) => next.nr_windows = Some(value),
            WorkloadDim::NrEvents(NrEventsSpec::Values(_)) => next.nr_events = Some(value),
            WorkloadDim::NrEvents(NrEventsSpec::Expr(_)) => {
                panic!("internal error: with_dim called for NrEventsExpr")
            }
            WorkloadDim::EventSpread(_) => next.event_spread = Some(value),
            WorkloadDim::EventOffset(_) => next.event_offset = Some(value),
            WorkloadDim::Bytes(_) => next.bytes = Some(value),
            WorkloadDim::Size(_) => next.size = Some(value),
            WorkloadDim::Slide(_) => next.slide = Some(value),
            WorkloadDim::Reserve(_) => next.reserve = Some(value),
        }
        next
    }

    fn finalize(self) -> Result<Workload, String> {
        let nr_windows = self.nr_windows.ok_or("missing workload dimension: nr_windows")?;
        let nr_events  = self.nr_events.ok_or("missing workload dimension: nr_events")?;
        let offset_u   = self.event_offset.ok_or("missing workload dimension: event_offset")?;
        let bytes      = self.bytes.ok_or("missing workload dimension: bytes")?;
        let size_u     = self.size.ok_or("missing workload dimension: size")?;
        let slide_u    = self.slide.ok_or("missing workload dimension: slide")?;
        let reserve    = self.reserve.unwrap_or(0);

        let spread_u = if self.event_spread_follows_slide {
            slide_u
        } else {
            self.event_spread.ok_or("missing workload dimension: event_spread")?
        };

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
            reserve,
        ))
    }
}

fn build_workloads_from_dims(dims: &[WorkloadDim]) -> Result<Vec<Workload>, String> {

    let mut seen = std::collections::HashSet::new();
    for dim in dims {
        let key = dim.key();
        if !seen.insert(key) {
            return Err(format!("duplicate workload dimension provided: {key}"));
        }
    }

    let mut partials = vec![Partial::new()];

    for dim in dims {
        match dim {
            WorkloadDim::NrWindows(values)
            | WorkloadDim::EventOffset(values)
            | WorkloadDim::Bytes(values)
            | WorkloadDim::Size(values)
            | WorkloadDim::Slide(values)
            | WorkloadDim::Reserve(values) => {
                let mut next_partials = Vec::with_capacity(partials.len() * values.len());
                for partial in &partials {
                    for &value in values {
                        next_partials.push(partial.with_dim(dim, value));
                    }
                }
                partials = next_partials;
            }
            WorkloadDim::NrEvents(NrEventsSpec::Values(values)) => {
                let mut next_partials = Vec::with_capacity(partials.len() * values.len());
                for partial in &partials {
                    for &value in values {
                        next_partials.push(partial.with_dim(dim, value));
                    }
                }
                partials = next_partials;
            }
            WorkloadDim::NrEvents(NrEventsSpec::Expr(expr)) => {
                let mut next_partials = Vec::with_capacity(partials.len());
                for partial in partials {
                    let value = eval_nr_events_expr(expr, &partial)?;
                    next_partials.push(partial.with_nr_events(value));
                }
                partials = next_partials;
            }
            WorkloadDim::EventSpread(EventSpreadSpec::Values(values)) => {
                let mut next_partials = Vec::with_capacity(partials.len() * values.len());
                for partial in &partials {
                    for &value in values {
                        next_partials.push(partial.with_dim(dim, value));
                    }
                }
                partials = next_partials;
            }
            WorkloadDim::EventSpread(EventSpreadSpec::FollowSlide) => {
                partials = partials
                    .into_iter()
                    .map(|p| p.with_event_spread_follow_slide())
                    .collect();
            }
        }
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
    let mut no_bench: Option<Strategy> = None;
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
            "--no-bench" => {
                i += 1;
                let value = all_args
                    .get(i)
                    .unwrap_or_else(|| panic!("expected a strategy after --no-bench"));
                let strategy = Strategy::parse(value)
                    .unwrap_or_else(|| panic!("unknown strategy for --no-bench: {}", value));
                no_bench = Some(strategy);
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
            | "--slide"
            | "--reserve") => {
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

    if no_bench.is_some() && !only.is_empty() {
        panic!("--only and --no-bench cannot be used together");
    }

    let workloads = build_workloads_from_dims(&workload_dims)
        .unwrap_or_else(|e| panic!("could not build workloads: {e}"));

    Args {
        folder_name,
        only: if only.is_empty() { None } else { Some(only) },
        no_bench,
        raw_command,
        workloads,
        sample_size,
    }
}

pub fn should_run(only: &Option<Vec<Strategy>>, strategy: Strategy) -> bool {
    match only {
        None => true,
        Some(list) => list.contains(&strategy),
    }
}

fn eval_nr_events_expr(expr: &str, partial: &Partial) -> Result<usize, String> {
    struct Parser<'a> {
        s: &'a [u8],
        i: usize,
    }

    impl<'a> Parser<'a> {
        fn new(input: &'a str) -> Self {
            Self {
                s: input.as_bytes(),
                i: 0,
            }
        }

        fn peek(&self) -> Option<u8> {
            self.s.get(self.i).copied()
        }

        fn bump(&mut self) {
            self.i += 1;
        }

        fn skip_ws(&mut self) {
            while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
                self.bump();
            }
        }

        fn eat(&mut self, ch: u8) -> bool {
            self.skip_ws();
            if self.peek() == Some(ch) {
                self.bump();
                true
            } else {
                false
            }
        }

        fn parse_expr(
            &mut self,
            partial: &Partial,
        ) -> Result<i64, String> {
            let mut lhs = self.parse_term(partial)?;
            loop {
                self.skip_ws();
                match self.peek() {
                    Some(b'+') => {
                        self.bump();
                        lhs += self.parse_term(partial)?;
                    }
                    Some(b'-') => {
                        self.bump();
                        lhs -= self.parse_term(partial)?;
                    }
                    _ => break,
                }
            }
            Ok(lhs)
        }

        fn parse_term(
            &mut self,
            partial: &Partial,
        ) -> Result<i64, String> {
            let mut lhs = self.parse_factor(partial)?;
            loop {
                self.skip_ws();
                match self.peek() {
                    Some(b'*') => {
                        self.bump();
                        lhs *= self.parse_factor(partial)?;
                    }
                    Some(b'/') => {
                        self.bump();
                        let rhs = self.parse_factor(partial)?;
                        if rhs == 0 {
                            return Err("division by zero in nr_events expression".to_string());
                        }
                        lhs /= rhs;
                    }
                    _ => break,
                }
            }
            Ok(lhs)
        }

        fn parse_factor(
            &mut self,
            partial: &Partial,
        ) -> Result<i64, String> {
            self.skip_ws();

            if self.eat(b'(') {
                let value = self.parse_expr(partial)?;
                if !self.eat(b')') {
                    return Err("missing closing ')' in nr_events expression".to_string());
                }
                return Ok(value);
            }

            if self.eat(b'-') {
                return Ok(-self.parse_factor(partial)?);
            }

            match self.peek() {
                Some(c) if c.is_ascii_digit() => self.parse_number(),
                Some(c) if c.is_ascii_alphabetic() || c == b'_' => self.parse_ident(partial),
                Some(c) => Err(format!(
                    "unexpected character '{}' in nr_events expression",
                    c as char
                )),
                None => Err("unexpected end of nr_events expression".to_string()),
            }
        }

        fn parse_number(&mut self) -> Result<i64, String> {
            self.skip_ws();
            let start = self.i;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
            }
            let text = std::str::from_utf8(&self.s[start..self.i]).unwrap();
            text.parse::<i64>()
                .map_err(|_| format!("invalid integer literal `{text}` in nr_events expression"))
        }

        fn parse_ident(
            &mut self,
            partial: &Partial,
        ) -> Result<i64, String> {
            self.skip_ws();
            let start = self.i;
            while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == b'_') {
                self.bump();
            }

            let ident = std::str::from_utf8(&self.s[start..self.i]).unwrap();

            let value = match ident {
                "slide" => partial
                    .slide
                    .ok_or_else(|| "nr_events expression uses `slide` before it is set".to_string())?,
                "offset" | "event_offset" => partial
                    .event_offset
                    .ok_or_else(|| "nr_events expression uses `offset` before it is set".to_string())?,
                "size" => partial
                    .size
                    .ok_or_else(|| "nr_events expression uses `size` before it is set".to_string())?,
                "bytes" => partial
                    .bytes
                    .ok_or_else(|| "nr_events expression uses `bytes` before it is set".to_string())?,
                "nr_windows" => partial
                    .nr_windows
                    .ok_or_else(|| "nr_events expression uses `nr_windows` before it is set".to_string())?,
                "reserve" => partial
                    .reserve
                    .ok_or_else(|| "nr_events expression uses `reserve` before it is set".to_string())?,
                other => {
                    return Err(format!(
                        "unknown variable `{other}` in nr_events expression"
                    ))
                }
            };

            Ok(value as i64)
        }
    }

    let mut p = Parser::new(expr);
    let value = p.parse_expr(partial)?;
    p.skip_ws();

    if p.i != p.s.len() {
        return Err("trailing input in nr_events expression".to_string());
    }

    if value < 0 {
        return Err(format!(
            "nr_events expression evaluated to negative value {value}"
        ));
    }

    usize::try_from(value)
        .map_err(|_| format!("nr_events expression result out of range: {value}"))
}