use criterion::{black_box, Criterion};
use reloaded_hooks_portable::api::jit::{
    compiler::{Jit, JitError},
    operation::Operation,
    operation_aliases::*,
};
use reloaded_hooks_x86_sys::x86::{jit::JitX86, Register};

pub(crate) fn benchmark_compile_only_x86(c: &mut Criterion) {
    let ops = create_operations_32();
    c.bench_function("assemble_x86_compile_only", |b| {
        b.iter(|| black_box(compile_instructions_32(0, &ops)))
    });
}

// Separate function for the code to be benchmarked.
#[allow(dead_code)]
pub(crate) fn create_and_assemble_instructions_32(
    address: usize,
) -> Result<Vec<u8>, JitError<Register>> {
    let operations = create_operations_32();
    compile_instructions_32(address, &operations)
}

pub(crate) fn create_operations_32() -> Vec<Operation<Register>> {
    let operations = vec![
        Operation::Push(Push {
            register: Register::eax,
        }),
        Operation::Mov(Mov {
            source: Register::eax,
            target: Register::ebx,
        }),
        Operation::StackAlloc(StackAlloc { operand: 10 }),
        Operation::Xchg(XChg {
            register1: Register::eax,
            register2: Register::ebx,
            scratch: None,
        }),
    ];
    operations
}

pub(crate) fn compile_instructions_32(
    address: usize,
    operations: &[Operation<Register>],
) -> Result<Vec<u8>, JitError<Register>> {
    JitX86::compile(address, operations)
}
