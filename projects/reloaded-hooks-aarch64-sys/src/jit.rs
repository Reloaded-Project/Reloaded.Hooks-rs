extern crate alloc;

use crate::{
    all_registers::AllRegisters,
    code_rewriter::aarch64_rewriter::is_b_or_bl,
    helpers::{vec_i32_to_u8, vec_u8_to_i32},
    instructions::b::B,
    jit_instructions::{
        branch_absolute::{encode_call_absolute, encode_jump_absolute},
        branch_ip_relative::{encode_call_ip_relative, encode_jump_ip_relative},
        branch_relative::{encode_call_relative, encode_jump_relative},
        jump_absolute_indirect::encode_jump_absolute_indirect,
        mov::encode_mov,
        mov_from_stack::encode_mov_from_stack,
        multi_pop::encode_multi_pop,
        multi_push::encode_multi_push,
        pop::encode_pop,
        push::encode_push,
        push_constant::encode_push_constant,
        push_stack::encode_push_stack,
        ret::encode_return,
        stackalloc::encode_stackalloc,
        xchg::encode_xchg,
    },
};
use alloc::vec::Vec;
use core::{
    mem::{self, size_of},
    ptr::read_unaligned,
};
use reloaded_hooks_portable::api::jit::{
    call_relative_operation::CallRelativeOperation,
    compiler::{Jit, JitCapabilities, JitError},
    jump_absolute_operation::JumpAbsoluteOperation,
    jump_relative_operation::JumpRelativeOperation,
    operation::Operation,
};

pub struct JitAarch64 {}

impl Jit<AllRegisters> for JitAarch64 {
    fn compile(
        address: usize,
        operations: &[Operation<AllRegisters>],
    ) -> Result<Vec<u8>, JitError<AllRegisters>> {
        // Initialize Assembler

        // Usually most opcodes will correspond to 1 instruction, however there may be 2
        // in some cases, so we reserve accordingly.

        // As all instructions are 32-bits in Aarch64, we use an i32 vec.
        let mut buf = Vec::with_capacity(operations.len() * 2 * size_of::<i32>());
        Self::compile_with_buf(address, operations, &mut buf)?;
        Ok(buf)
    }

    fn compile_with_buf(
        address: usize,
        operations: &[Operation<AllRegisters>],
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<AllRegisters>> {
        let mut pc = address;
        let mut buf_i32 = vec_u8_to_i32(mem::take(buf));
        for operation in operations {
            encode_instruction_aarch64(operation, &mut pc, &mut buf_i32)?;
        }

        *buf = vec_i32_to_u8(buf_i32);
        Ok(())
    }

    fn code_alignment() -> u32 {
        4
    }

    fn max_relative_jump_distances() -> &'static [usize] {
        // We remove a -4 value because forward jumps can't go as far.
        &[
            (1024 * 1024) - 4,        // -+ 1 MiB (Branch Conditional)
            (1024 * 1024 * 128) - 4,  // -+ 128 MiB (Branch)
            (1024 * 1024 * 4096) - 4, // -+ 4 GiB
        ]
    }

    fn get_jit_capabilities() -> JitCapabilities {
        JitCapabilities::CAN_MULTI_PUSH
            | JitCapabilities::CAN_ENCODE_IP_RELATIVE_CALL
            | JitCapabilities::CAN_ENCODE_IP_RELATIVE_JUMP
    }

    fn max_branch_bytes() -> u32 {
        24 // MOVZ + MOVK + LDR + BR
    }

    fn fill_nops(arr: &mut [u8]) {
        const NOP: [u8; 4] = [0xD5, 0x03, 0x20, 0x1F];

        // Ensure the array length is a multiple of 4 (size of an ARM64 instruction)
        for chunk in arr.chunks_mut(4) {
            chunk.copy_from_slice(&NOP);
        }
    }

    fn encode_jump(
        x: &JumpRelativeOperation<AllRegisters>,
        pc: &mut usize,
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<AllRegisters>> {
        let mut buf_i32 = vec_u8_to_i32(mem::take(buf));
        let result = encode_jump_relative(x, pc, &mut buf_i32);
        *buf = vec_i32_to_u8(buf_i32);
        result
    }

    fn max_relative_jump_bytes() -> usize {
        12
    }

    fn encode_call(
        x: &CallRelativeOperation,
        pc: &mut usize,
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<AllRegisters>> {
        let mut buf_i32 = vec_u8_to_i32(mem::take(buf));
        let result = encode_call_relative(x, pc, &mut buf_i32);
        *buf = vec_i32_to_u8(buf_i32);
        result
    }

    fn decode_call_target(ins_address: usize, ins_length: usize) -> Result<usize, &'static str> {
        if ins_length < 4 {
            return Err("[ARM64: decode_call_target] Instruction is too short.");
        }

        // Need to do from BE for some reason.
        let num: u32 = u32::from_le(unsafe { read_unaligned(ins_address as *const u32) });
        let instruction = B(num);

        if !is_b_or_bl(instruction.0) {
            return Err("[ARM64: decode_call_target] This is not a branch instruction.");
        }

        if !instruction.is_link() {
            return Err("[ARM64: decode_call_target] This is a branch, but not 'branch with link' instruction.");
        }

        Ok((ins_address as isize).wrapping_add(instruction.offset() as isize) as usize)
    }

    fn encode_abs_jump(
        x: &JumpAbsoluteOperation<AllRegisters>,
        pc: &mut usize,
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<AllRegisters>> {
        let mut buf_i32 = vec_u8_to_i32(mem::take(buf));
        let result = encode_jump_absolute(x, pc, &mut buf_i32);
        *buf = vec_i32_to_u8(buf_i32);
        result
    }

    fn max_standard_relative_call_distance() -> usize {
        (1024 * 1024 * 128) - 4
    }

    fn standard_relative_call_bytes() -> usize {
        4
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
        Operation::JumpAbsoluteIndirect(x) => encode_jump_absolute_indirect(x, pc, buf),
        Operation::Return(x) => encode_return(x, pc, buf),
        Operation::CallIpRelative(x) => encode_call_ip_relative(x, pc, buf),
        Operation::JumpIpRelative(x) => encode_jump_ip_relative(x, pc, buf),
        Operation::MultiPush(x) => encode_multi_push(x, pc, buf),
        Operation::MultiPop(x) => encode_multi_pop(x, pc, buf),
    }
}
