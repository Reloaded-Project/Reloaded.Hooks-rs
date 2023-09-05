use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};
use wrapper_instruction_generator::benchmark_wrappers;
mod wrapper_instruction_generator;

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_wrappers(c);
}

#[cfg(not(target_os = "windows"))]
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark
}

criterion_main!(benches);
