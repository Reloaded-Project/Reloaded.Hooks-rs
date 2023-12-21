mod benchmarks;
use benchmarks::{
    assembler_bench_64::{benchmark_assemble_x64_total, benchmark_compile_only},
    assembly_hook_bench_64::benchmark_create_assembly_hook,
};
use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_assemble_x64_total(c); // assemble_x64_total
    benchmark_compile_only(c); // assemble_x64_compile_only

    // Flawed benchmark, see readme!!
    // benchmark_create_assembly_hook(c); // assembly_hook_creation
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
