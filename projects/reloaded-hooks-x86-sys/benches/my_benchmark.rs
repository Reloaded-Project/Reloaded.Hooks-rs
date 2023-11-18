mod assembler_bench_64;
use assembler_bench_64::{
    compile_instructions_64, create_and_assemble_instructions_64, create_operations_64,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};
use reloaded_hooks_x86_sys::x64::jit::JitX64;

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_assemble_x64_total(c);
    benchmark_compile_only(c);
}

fn benchmark_compile_only(c: &mut Criterion) {
    let mut jit = JitX64 {};
    let ops = create_operations_64();
    c.bench_function("assemble_x64_compile_only", |b| {
        b.iter(|| black_box(compile_instructions_64(&mut jit, 0, &ops)))
    });
}

fn benchmark_assemble_x64_total(c: &mut Criterion) {
    c.bench_function("assemble_x64_total", |b| {
        b.iter(|| black_box(create_and_assemble_instructions_64(0)))
    });
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
