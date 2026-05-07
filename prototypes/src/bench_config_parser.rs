use serde::Deserialize;
use std::{env, fs, path::{Path, PathBuf}};

#[derive(Debug, Deserialize)]
struct BenchConfig {
    output: Option<OutputConfig>,
}

#[derive(Debug, Deserialize)]
struct OutputConfig {
    root: Option<PathBuf>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .expect("crate dir has no parent; expected repo root above crate")
}

fn bench_config_path() -> PathBuf {
    repo_root().join("bench_config.toml")
}

fn default_output_root() -> PathBuf {
    repo_root().join("analysis").join("evaluation")
}

fn load_bench_config() -> BenchConfig {
    let path = bench_config_path();

    if !path.exists() {
        return BenchConfig { output: None };
    }

    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read config file {:?}: {e}", path));

    toml::from_str(&text)
        .unwrap_or_else(|e| panic!("failed to parse TOML config {:?}: {e}", path))
}

pub fn resolve_output_root() -> PathBuf {
    let config = load_bench_config();

    match config.output.and_then(|o| o.root) {
        Some(path) => {
            if !path.is_absolute() {
                panic!(
                    "configured output.root must be an absolute path, got {:?}",
                    path
                );
            }
            path
        }
        None => default_output_root(),
    }
}