extern crate alloc;

use reloaded_hooks_portable::api::jit::{
    compiler::{Jit, JitCapabilities, JitError},
    operation::Operation,
};

use alloc::rc::Rc;
use alloc::vec::Vec;
use core::{mem::size_of, slice};

use crate::{
    all_registers::AllRegisters,
    jit_instructions::{
        branch_absolute::{encode_call_absolute, encode_jump_absolute},
        branch_relative::{encode_call_relative, encode_jump_relative},
        mov::encode_mov,
        mov_from_stack::encode_mov_from_stack,
        pop::encode_pop,
        push::encode_push,
        push_constant::encode_push_constant,
        push_stack::encode_push_stack,
        stackalloc::encode_stackalloc,
        xchg::encode_xchg,
    },
};

pub struct JitAarch64 {}

impl Jit<AllRegisters> for JitAarch64 {
    fn compile(
        &mut self,
        address: usize,
        operations: &[Operation<AllRegisters>],
    ) -> Result<Rc<[u8]>, JitError<AllRegisters>> {
        // Initialize Assembler

        // Usually most opcodes will correspond to 1 instruction, however there may be 2
        // in some cases, so we reserve accordingly.

        // As all instructions are 32-bits in Aarch64, we use an i32 vec.
        let mut buf = Vec::<i32>::with_capacity(operations.len() * 2);
        let mut pc = address;

        // Encode every instruction.
        for operation in operations {
            encode_instruction_aarch64(operation, &mut pc, &mut buf)?;
        }

        let ptr = buf.as_ptr() as *const u8;
        unsafe {
            let slice = slice::from_raw_parts(ptr, buf.len() * size_of::<i32>());
            Ok(Rc::from(slice))
        }
    }

    fn code_alignment() -> u32 {
        4
    }

    fn max_relative_jump_distance() -> usize {
        (1024 * 1024 * 128) - 1 // -+ 128 MiB (-1 for forward jump)
    }

    fn get_jit_capabilities() -> &'static [JitCapabilities] {
        &[
            // JitCapabilities::CanMultiPush,            // Not currently implemented. Possible.
            // JitCapabilities::CanEncodeIPRelativeCall, // (Possible with ADR, just not currently implemented)
            // JitCapabilities::CanEncodeIPRelativeJump, // (Possible with ADR, just not currently implemented)
        ]
    }
}

fn encode_instruction_aarch64(
    operation: &Operation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    match operation {
        Operation::None => Ok(()),
        Operation::Mov(x) => encode_mov(x, pc, buf),
        Operation::MovFromStack(x) => encode_mov_from_stack(x, pc, buf),
        Operation::Push(x) => encode_push(x, pc, buf),
        Operation::PushStack(x) => encode_push_stack(x, pc, buf),
        Operation::PushConst(x) => encode_push_constant(x, pc, buf),
        Operation::StackAlloc(x) => encode_stackalloc(x, pc, buf),
        Operation::Pop(x) => encode_pop(x, pc, buf),
        Operation::Xchg(x) => encode_xchg(x, pc, buf),
        Operation::CallAbsolute(x) => encode_call_absolute(x, pc, buf),
        Operation::CallRelative(x) => encode_call_relative(x, pc, buf),
        Operation::JumpRelative(x) => encode_jump_relative(x, pc, buf),
        Operation::JumpAbsolute(x) => encode_jump_absolute(x, pc, buf),
        Operation::Return(_) => todo!(),
        Operation::CallIpRelative(_) => todo!(), // Not implementable
        Operation::JumpIpRelative(_) => todo!(), // Not implementable
        Operation::MultiPush(_) => todo!(),
        Operation::MultiPop(_) => todo!(),
    }
}
