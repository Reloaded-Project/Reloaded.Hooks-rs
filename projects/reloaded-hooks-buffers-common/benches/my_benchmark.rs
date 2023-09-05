use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};
use reloaded_hooks_x86_sys::x64::jit::JitX64;

fn criterion_benchmark(c: &mut Criterion) {}

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
