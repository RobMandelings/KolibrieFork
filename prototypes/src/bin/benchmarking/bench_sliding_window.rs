use criterion::{black_box, BenchmarkGroup, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::env;
use criterion::measurement::WallTime;
// use your crate types
use prototypes::workloads::{default_workloads, Workload};
use prototypes::{
    run_strategy_arc, run_strategy_expire, run_strategy_refcount, WindowParams,
};

struct ConfigGrid<'a> {
    event_counts: &'a [usize],
    window_configs: &'a [WindowParams],
}

pub(crate) fn bench_strategy<F>(
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

fn bench_sliding_window(c: &mut Criterion, workloads: &Vec<Workload>) {

    for workload in workloads {

        // One group per workload
        let mut group = c.benchmark_group(&workload.name);
        bench_strategy(&mut group, "ExpireStrategy", workload, |workload| {
            run_strategy_expire(workload)
        });

        bench_strategy(&mut group, "CloneStrategy", workload, |workload| {
            run_strategy_arc(workload)
        });

        bench_strategy(&mut group, "RefCountStrategy", workload, |workload| {
            run_strategy_refcount(workload)
        });
        group.finish();
    }
}

fn parse_config_string(s: &str) -> Option<WindowParams> {
    // expect "size=...,slide=...,offset=..."
    let mut map = HashMap::new();
    for part in s.split(',') {
        let mut it = part.splitn(2, '=');
        let key = it.next()?.trim();
        let val = it.next()?.trim();
        map.insert(key, val);
    }

    let size = map.get("size")?.parse().ok()?;
    let slide = map.get("slide")?.parse().ok()?;
    let offset = map.get("offset")?.parse().ok()?;

    Some(WindowParams {
        size,
        slide,
        offset,
    })
}

fn main() {

    let args: Vec<String> = env::args().skip(1).collect();
    eprintln!("args = {:?}", args);

    let mut c: Criterion = Criterion::default().with_output_color(true);
    let workloads = default_workloads();

    // run your benchmarks once
    bench_sliding_window(&mut c, &workloads);

    // finalize reports
    c.final_summary();
}

// criterion_group!(benches, bench_sliding_window);
// criterion_main!(benches);
