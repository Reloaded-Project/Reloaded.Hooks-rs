extern crate alloc;
use reloaded_hooks_portable::api::jit::{
    compiler::{Jit, JitCapabilities, JitError},
    jump_relative_operation::JumpRelativeOperation,
    operation::Operation,
};

use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::{mem::size_of, slice};

use crate::all_registers::AllRegisters;

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
        Operation::Mov(_) => todo!(),
        Operation::MovFromStack(_) => todo!(),
        Operation::Push(_) => todo!(),
        Operation::PushStack(_) => todo!(),
        Operation::PushConst(_) => todo!(),
        Operation::StackAlloc(_) => todo!(),
        Operation::Pop(_) => todo!(),
        Operation::Xchg(_) => todo!(),
        Operation::CallAbsolute(_) => todo!(),
        Operation::CallRelative(_) => todo!(),
        Operation::JumpRelative(x) => encode_jump_relative(x, pc, buf),
        Operation::JumpAbsolute(_) => todo!(),
        Operation::Return(_) => todo!(),
        Operation::CallIpRelative(_) => todo!(),
        Operation::JumpIpRelative(_) => todo!(),
        Operation::MultiPush(_) => todo!(),
        Operation::MultiPop(_) => todo!(),
    }
}

fn encode_jump_relative(
    x: &JumpRelativeOperation,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    // Branch uses number of 4 byte instructions to jump, so we divide by 4.
    // i.e. value of 1 jumps 4 bytes.
    let offset = (x.target_address as i32 - *pc as i32) >> 2;

    if !(-0x02000000..=0x01FFFFFF).contains(&offset) {
        return Err(JitError::OperandOutOfRange(
            "Jump distance for Branch Instruction specified in JumpRelativeOperation too great"
                .to_string(),
        ));
    }

    // Convert the 32-bit offset into a 26-bit offset by shifting right 6 bits
    let imm26 = offset & 0x03FFFFFF;

    // Create the instruction encoding for the B instruction
    let instruction = 0b000101 << 26 | imm26;

    *pc += 4;
    buf.push(instruction.to_le());

    Ok(())
}

#[cfg(test)]

mod tests {
    use core::{mem::size_of_val, slice};

    use reloaded_hooks_portable::api::jit::jump_relative_operation::JumpRelativeOperation;
    use rstest::rstest;

    use crate::jit::encode_jump_relative;

    fn instruction_buffer_as_hex(buf: &[i32]) -> String {
        let ptr = buf.as_ptr() as *const u8;
        unsafe {
            let as_u8 = slice::from_raw_parts(ptr, size_of_val(buf));
            hex::encode(as_u8)
        }
    }

    #[rstest]
    #[case(0, 4, "01000014", 4)] // jump forward
    #[case(4, 0, "ffffff17", 8)] // jump backward
    #[case(4, 4, "00000014", 8)] // no jump
    fn test_encode_jump_relative(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
        #[case] expected_pc: usize,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        let operation = JumpRelativeOperation { target_address };

        assert!(encode_jump_relative(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_pc, pc);
    }

    #[test]
    fn test_encode_jump_relative_invalid_forward() {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = JumpRelativeOperation {
            target_address: 1024 * 1024 * 128,
        }; // some invalid value

        assert!(encode_jump_relative(&operation, &mut pc, &mut buf).is_err());
        // Add additional checks for the state of `buf` and `pc` here.
    }

    #[rstest]
    #[case(0, 1024 * 1024 * 128)] // Invalid forward jump
    #[case(1024 * 1024 * 128 + 1, 0)] // Invalid backward jump
    fn test_encode_jump_relative_invalid(#[case] initial_pc: usize, #[case] target_address: usize) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        let operation = JumpRelativeOperation { target_address };

        assert!(encode_jump_relative(&operation, &mut pc, &mut buf).is_err());
        // Add additional checks for the state of `buf` and `pc` here.
    }
}
