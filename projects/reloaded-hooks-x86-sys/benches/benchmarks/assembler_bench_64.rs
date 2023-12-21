use criterion::{black_box, Criterion};
use reloaded_hooks_portable::api::jit::{
    compiler::{Jit, JitError},
    operation::Operation,
    operation_aliases::*,
};
use reloaded_hooks_x86_sys::x64::{self, jit::JitX64, Register};

pub(crate) fn benchmark_compile_only(c: &mut Criterion) {
    let ops = create_operations_64();
    c.bench_function("assemble_x64_compile_only", |b| {
        b.iter(|| black_box(compile_instructions_64(0, &ops)))
    });
}

pub(crate) fn benchmark_assemble_x64_total(c: &mut Criterion) {
    c.bench_function("assemble_x64_total", |b| {
        b.iter(|| black_box(create_and_assemble_instructions_64(0)))
    });
}

// Separate function for the code to be benchmarked.
#[allow(dead_code)]
pub(crate) fn create_and_assemble_instructions_64(
    address: usize,
) -> Result<Vec<u8>, JitError<x64::Register>> {
    let operations = create_operations_64();
    compile_instructions_64(address, &operations)
}

pub(crate) fn create_operations_64() -> Vec<Operation<Register>> {
    let operations = vec![
        Operation::Push(Push {
            register: Register::rax,
        }),
        Operation::Mov(Mov {
            source: Register::rax,
            target: Register::rbx,
        }),
        Operation::StackAlloc(StackAlloc { operand: 10 }),
        Operation::Xchg(XChg {
            register1: Register::rax,
            register2: Register::rbx,
            scratch: None,
        }),
    ];
    operations
}

pub(crate) fn compile_instructions_64(
    address: usize,
    operations: &[Operation<Register>],
) -> Result<Vec<u8>, JitError<Register>> {
    JitX64::compile(address, operations)
}
