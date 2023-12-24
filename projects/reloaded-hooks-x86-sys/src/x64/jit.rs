// JIT for x64
extern crate alloc;

use crate::common::jit_common::encode_instruction;
use crate::common::jit_conversions_common::{
    map_allregisters_to_x64, map_register_x64_to_allregisters,
};
use crate::common::jit_instructions::decode_relative_call_target::decode_call_target;
use crate::common::jit_instructions::encode_absolute_jump::encode_absolute_jump_x64;
use crate::common::jit_instructions::encode_relative_call::encode_call_relative;
use crate::common::jit_instructions::encode_relative_jump::encode_jump_relative;
use crate::x64::register::Register;
use alloc::{string::ToString, vec::Vec};
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::call_relative_operation::CallRelativeOperation;
use reloaded_hooks_portable::api::jit::compiler::Jit;
use reloaded_hooks_portable::api::jit::jump_absolute_operation::JumpAbsoluteOperation;
use reloaded_hooks_portable::api::jit::jump_relative_operation::JumpRelativeOperation;
use reloaded_hooks_portable::api::jit::{
    compiler::{transform_err, JitCapabilities, JitError},
    operation::{transform_op, Operation},
};

pub struct JitX64 {}

/// Implementation of the x64 JIT.
impl Jit<Register> for JitX64 {
    fn compile(
        address: usize,
        operations: &[Operation<Register>],
    ) -> Result<Vec<u8>, JitError<Register>> {
        // Initialize Assembler
        let mut a = CodeAssembler::new(64)
            .map_err(|x| JitError::CannotInitializeAssembler(x.to_string()))?;

        // Encode every instruction.
        for operation in operations {
            encode_instruction_x64(&mut a, operation, address)?;
        }

        // Assemble those damn instructions
        a.assemble(address as u64)
            .map_err(|x| JitError::CannotInitializeAssembler(x.to_string()))
    }

    fn compile_with_buf(
        address: usize,
        operations: &[Operation<Register>],
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<Register>> {
        // Initialize Assembler
        let mut a = CodeAssembler::new(64)
            .map_err(|x| JitError::CannotInitializeAssembler(x.to_string()))?;

        // Encode every instruction.
        for operation in operations {
            encode_instruction_x64(&mut a, operation, address)?;
        }

        // Assemble those damn instructions
        let result = a
            .assemble(address as u64)
            .map_err(|x| JitError::CannotInitializeAssembler(x.to_string()))?;

        buf.extend(result);
        Ok(())
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
            | JitCapabilities::CAN_MULTI_PUSH
            | JitCapabilities::PROFITABLE_ABSOLUTE_INDIRECT_JUMP
    }

    fn max_branch_bytes() -> u32 {
        12 // mov <reg>, address + call .
    }

    fn max_indirect_offsets() -> &'static [u32] {
        &[0x7FFFFFFF, 0xFFFFFFFF]
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

    fn decode_call_target(ins_address: usize, ins_length: usize) -> Result<usize, &'static str> {
        decode_call_target(ins_address, ins_length)
    }

    fn encode_abs_jump(
        x: &JumpAbsoluteOperation<Register>,
        _pc: &mut usize,
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<Register>> {
        encode_absolute_jump_x64(x, buf)
    }

    fn max_standard_relative_call_distance() -> usize {
        i32::MAX as usize
    }

    fn standard_relative_call_bytes() -> usize {
        5
    }
}

fn encode_instruction_x64(
    assembler: &mut CodeAssembler,
    operation: &Operation<Register>,
    address: usize,
) -> Result<(), JitError<Register>> {
    let all_register_op = transform_op(operation.clone(), |x: Register| {
        map_register_x64_to_allregisters(x)
    });

    encode_instruction(assembler, &all_register_op, address)
        .map_err(|x| transform_err(x, map_allregisters_to_x64))
}
