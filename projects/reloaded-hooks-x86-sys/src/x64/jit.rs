// JIT for x64
extern crate alloc;

use crate::common::jit_common::encode_instruction;
use crate::common::jit_conversions_common::{
    map_allregisters_to_x64, map_register_x64_to_allregisters,
};
use crate::x64::register::Register;
use alloc::{rc::Rc, string::ToString};
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{
    compiler::{transform_err, Jit, JitCapabilities, JitError},
    operation::{transform_op, Operation},
};

pub struct JitX64 {}

/// Implementation of the x64 JIT.
impl Jit<Register> for JitX64 {
    fn compile(
        address: usize,
        operations: &[Operation<Register>],
    ) -> Result<Rc<[u8]>, JitError<Register>> {
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

        Ok(Rc::from(result))
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
