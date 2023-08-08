use std::rc::Rc;

use reloaded_hooks_portable::api::jit::compiler::Jit;
use reloaded_hooks_portable::api::jit::{
    compiler::JitError, mov_operation::MovOperation, operation::Operation,
    push_operation::PushOperation, sub_operation::SubOperation, xchg_operation::XChgOperation,
};
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
        Operation::Push(PushOperation {
            register: Register::rax,
        }),
        Operation::Mov(MovOperation {
            source: Register::rax,
            target: Register::rbx,
        }),
        Operation::Sub(SubOperation {
            register: Register::rax,
            operand: 10,
        }),
        Operation::Xchg(XChgOperation {
            register1: Register::rax,
            register2: Register::rbx,
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
