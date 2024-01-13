// JIT for x86
extern crate alloc;

use crate::common::jit_instructions::decode_relative_call_target::decode_call_target;
use crate::instructions::call_absolute::encode_absolute_call_x86;
use crate::instructions::call_relative::encode_call_relative;
use crate::instructions::jump_absolute::encode_absolute_jump_x86;
use crate::instructions::jump_absolute_indirect::{
    encode_jump_absolute_indirect_x64, encode_jump_absolute_indirect_x86,
};
use crate::instructions::jump_relative::encode_jump_relative;
use crate::instructions::mov::encode_mov_x86;
use crate::instructions::mov_from_stack::encode_mov_from_stack_x86;
use crate::instructions::mov_to_stack::encode_mov_to_stack_x86;
use crate::instructions::pop::encode_pop_32;
use crate::instructions::push::encode_push_32;
use crate::instructions::push_const::encode_push_constant_x86;
use crate::instructions::push_stack::encode_push_stack_x86;
use crate::instructions::ret::encode_return;
use crate::instructions::stack_alloc::encode_stack_alloc_32;
use crate::instructions::xchg::encode_xchg_x86;
use crate::x86::register::Register;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::call_relative_operation::CallRelativeOperation;
use reloaded_hooks_portable::api::jit::compiler::DecodeCallTargetResult;
use reloaded_hooks_portable::api::jit::jump_absolute_operation::JumpAbsoluteOperation;
use reloaded_hooks_portable::api::jit::jump_relative_operation::JumpRelativeOperation;
use reloaded_hooks_portable::api::jit::{
    compiler::{Jit, JitCapabilities, JitError},
    operation::Operation,
};

pub struct JitX86 {}

/// Implementation of the x86 JIT.
impl Jit<Register> for JitX86 {
    fn compile(
        address: usize,
        operations: &[Operation<Register>],
    ) -> Result<Vec<u8>, JitError<Register>> {
        let mut vec = Vec::with_capacity(operations.len() * 4);
        Self::compile_with_buf(address, operations, &mut vec)?;
        Ok(vec)
    }

    fn compile_with_buf(
        address: usize,
        operations: &[Operation<Register>],
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<Register>> {
        // Encode every instruction.
        let mut pc = address;
        for operation in operations {
            encode_instruction_x86(operation, &mut pc, buf)?;
        }
        Ok(())
    }

    fn stack_entry_misalignment() -> u32 {
        // Note: x86 does not impose alignment requirements on stack
        // If user runs across a custom convention with an alignment requirement
        // they should create 'shadow space' in their custom calling convention declaration.

        // This value is still however needed in any case, for the wrapper generator.
        4
    }

    fn code_alignment() -> u32 {
        16
    }

    fn max_relative_jump_distances() -> &'static [usize] {
        &[i32::MAX as usize]
    }

    fn get_jit_capabilities() -> JitCapabilities {
        JitCapabilities::CAN_ENCODE_IP_RELATIVE_CALL
            | JitCapabilities::CAN_ENCODE_IP_RELATIVE_JUMP
            | JitCapabilities::CAN_MOV_TO_STACK
    }

    fn max_branch_bytes() -> u32 {
        5 // jmp/call rel32
    }

    fn fill_nops(arr: &mut [u8]) {
        for byte in arr.iter_mut() {
            *byte = 0x90;
        }
    }

    fn encode_jump(
        x: &JumpRelativeOperation<Register>,
        pc: &mut usize,
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<Register>> {
        encode_jump_relative(x, pc, buf)
    }

    fn max_relative_jump_bytes() -> usize {
        5
    }

    fn encode_call(
        x: &CallRelativeOperation,
        pc: &mut usize,
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<Register>> {
        encode_call_relative(x, pc, buf)
    }

    fn decode_call_target(
        ins_address: usize,
        ins_length: usize,
    ) -> Result<DecodeCallTargetResult, &'static str> {
        decode_call_target(ins_address, ins_length)
    }

    fn encode_abs_jump(
        x: &JumpAbsoluteOperation<Register>,
        pc: &mut usize,
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<Register>> {
        encode_absolute_jump_x86(x, pc, buf)
    }

    fn max_standard_relative_call_distance() -> usize {
        i32::MAX as usize
    }

    fn standard_relative_call_bytes() -> usize {
        5
    }

    fn standard_register_size() -> usize {
        4
    }
}

pub(crate) fn encode_instruction_x86(
    operation: &Operation<Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), JitError<Register>> {
    match operation {
        Operation::Mov(x) => Ok(encode_mov_x86(x, pc, buf)?),
        Operation::MovFromStack(x) => Ok(encode_mov_from_stack_x86(x, pc, buf)?),
        Operation::Push(x) => Ok(encode_push_32(x, pc, buf)?),
        Operation::PushStack(x) => Ok(encode_push_stack_x86(x, pc, buf)?),
        Operation::StackAlloc(x) => Ok(encode_stack_alloc_32(x, pc, buf)?),
        Operation::Pop(x) => Ok(encode_pop_32(x, pc, buf)?),
        Operation::Xchg(x) => Ok(encode_xchg_x86(x, pc, buf)?),
        Operation::CallRelative(x) => Ok(encode_call_relative(x, pc, buf)?),
        Operation::CallAbsolute(x) => Ok(encode_absolute_call_x86(x, pc, buf)?),
        Operation::JumpRelative(x) => Ok(encode_jump_relative(x, pc, buf)?),
        Operation::JumpAbsolute(x) => Ok(encode_absolute_jump_x86(x, pc, buf)?),
        Operation::JumpAbsoluteIndirect(x) => Ok(encode_jump_absolute_indirect_x86(x, pc, buf)?),
        Operation::MovToStack(x) => Ok(encode_mov_to_stack_x86(x, pc, buf)?),
        Operation::PushConst(x) => Ok(encode_push_constant_x86(x, pc, buf)?),
        Operation::Return(x) => Ok(encode_return(x, pc, buf)?),
        _ => unreachable!(),
    }
}
