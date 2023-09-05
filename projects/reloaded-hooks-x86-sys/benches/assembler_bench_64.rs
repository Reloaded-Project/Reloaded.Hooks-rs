use std::rc::Rc;

use reloaded_hooks_portable::api::jit::compiler::Jit;
use reloaded_hooks_portable::api::jit::operation_aliases::*;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation::Operation};
use reloaded_hooks_x86_sys::x64;
use reloaded_hooks_x86_sys::x64::jit::JitX64;
use reloaded_hooks_x86_sys::x64::Register;

// Separate function for the code to be benchmarked.
#[allow(dead_code)]
pub(crate) fn create_and_assemble_instructions_64(
    address: usize,
) -> Result<Rc<[u8]>, JitError<x64::Register>> {
    let mut jit = JitX64 {};
    let operations = create_operations_64();
    compile_instructions_64(&mut jit, address, &operations)
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
    jit: &mut JitX64,
    address: usize,
    operations: &[Operation<Register>],
) -> Result<Rc<[u8]>, JitError<Register>> {
    jit.compile(address, operations)
}
