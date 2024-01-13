mod benchmarks;
#[allow(unused_imports)]
use benchmarks::assembly_hook_bench_64::benchmark_create_assembly_hook; // commented out
use benchmarks::{
    assembler_bench_32::benchmark_compile_only_x86,
    assembler_bench_64::{benchmark_assemble_x64_total, benchmark_compile_only},
};
use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_assemble_x64_total(c); // assemble_x64_total
    benchmark_compile_only(c); // assemble_x64_compile_only
    benchmark_compile_only_x86(c); // for comparison with x64

    // at time of writing, x86 is 16x faster than x64 due to being custom implementation.
    // we are not writing a custom impl for x64 right now however, because it's a cold path
    // in x64; calling convention conversion is rare, so I'd rather save on binary size instead.

    // however, if anyone is motivated, feel free to write custom x64 impl based on the x86 one.

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
