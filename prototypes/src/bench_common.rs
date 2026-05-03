use std::{env, fs, io};
use std::path::Path;
use log::{debug, info};

pub fn move_profile_file(strat: &str, group_path: &Path) {
    let dir = group_path.join("memory");
    // create mem_profiles/workload_name if needed
    fs::create_dir_all(&dir).expect("failed to create mem_profiles dir");
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

pub struct Args {
    pub folder_name: String,
    pub only: Option<Vec<Strategy>>,
}

pub fn parse_args() -> Args {
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

pub fn should_run(only: &Option<Vec<Strategy>>, strategy: Strategy) -> bool {
    match only {
        None => true,
        Some(list) => list.contains(&strategy),
    }
}