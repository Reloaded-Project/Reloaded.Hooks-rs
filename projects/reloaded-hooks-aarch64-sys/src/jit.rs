extern crate alloc;
use reloaded_hooks_portable::api::jit::{
    call_relative_operation::CallRelativeOperation,
    compiler::{Jit, JitCapabilities, JitError},
    jump_relative_operation::JumpRelativeOperation,
    mov_operation::MovOperation,
    operation::Operation,
    operation_aliases::MovFromStack,
    pop_operation::PopOperation,
    push_operation::PushOperation,
    stack_alloc_operation::StackAllocOperation,
};

use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::{mem::size_of, slice};

use crate::{
    all_registers::AllRegisters,
    opcodes::{
        add_immediate::AddImmediate, ldr_immediate_post_indexed::LdrImmediatePostIndexed,
        ldr_immediate_unsigned_offset::LdrImmediateUnsignedOffset, orr::Orr,
        str_immediate_pre_indexed::StrImmediatePreIndexed, sub_immediate::SubImmediate,
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
        Operation::PushStack(x) => todo!(),
        Operation::PushConst(_) => todo!(),
        Operation::StackAlloc(x) => encode_stackalloc(x, pc, buf),
        Operation::Pop(x) => encode_pop(x, pc, buf),
        Operation::Xchg(_) => todo!(),
        Operation::CallAbsolute(_) => todo!(),
        Operation::CallRelative(x) => encode_call_relative(x, pc, buf),
        Operation::JumpRelative(x) => encode_jump_relative(x, pc, buf),
        Operation::JumpAbsolute(_) => todo!(),
        Operation::Return(_) => todo!(),
        Operation::CallIpRelative(_) => todo!(),
        Operation::JumpIpRelative(_) => todo!(),
        Operation::MultiPush(_) => todo!(),
        Operation::MultiPop(_) => todo!(),
    }
}

/// Encoded as LDR
fn encode_pop(
    x: &PopOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let size = x.register.size();
    if size == 8 {
        let ldr = LdrImmediatePostIndexed::new_pop_register(true, x.register as u8, 8)?;
        buf.push(ldr.0.to_le() as i32);
        *pc += 4;
        Ok(())
    } else if size == 4 {
        let ldr = LdrImmediatePostIndexed::new_pop_register(false, x.register as u8, 4)?;
        buf.push(ldr.0.to_le() as i32);
        *pc += 4;
        Ok(())
    } else if size == 16 {
        return encode_pop_vector(x, pc, buf);
    } else {
        return Err(JitError::InvalidRegister(x.register));
    }
}

fn encode_pop_vector(
    x: &PopOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    todo!()
}

/// Encoded as STR
fn encode_push(
    x: &PushOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let size = x.register.size();
    if size == 8 {
        let str = StrImmediatePreIndexed::new_push_register(true, x.register as u8, -8)?;
        buf.push(str.0.to_le() as i32);
        *pc += 4;
        Ok(())
    } else if size == 4 {
        let str = StrImmediatePreIndexed::new_push_register(false, x.register as u8, -4)?;
        buf.push(str.0.to_le() as i32);
        *pc += 4;
        Ok(())
    } else if size == 16 {
        return encode_push_vector(x, pc, buf);
    } else {
        return Err(JitError::InvalidRegister(x.register));
    }
}

fn encode_push_vector(
    x: &PushOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    todo!()
}

/// Encoded as SUB, SP, SP, #operand
/// or ADD, SP, SP, #operand
fn encode_stackalloc(
    x: &StackAllocOperation,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    if x.operand >= 0 {
        let sub = SubImmediate::new_stackalloc(true, x.operand as u16)?;

        *pc += 4;
        buf.push(sub.0.to_le() as i32);
        Ok(())
    } else {
        let add = AddImmediate::new_stackalloc(true, -x.operand as u16)?;

        *pc += 4;
        buf.push(add.0.to_le() as i32);
        Ok(())
    }
}

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDR--immediate---Load-Register--immediate--?lang=en#iclass_post_indexed
fn encode_mov_from_stack(
    x: &MovFromStack<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let target_size = x.target.size();

    // TODO: Handle Vector Registers
    // a.k.a. `sf` flag
    let is_64bit = if target_size == 8 {
        true
    } else if target_size == 4 {
        false
    } else if target_size == 16 {
        return encode_mov_from_stack_vector(x, pc, buf);
    } else {
        return Err(JitError::InvalidRegister(x.target));
    };

    let rd = x.target.register_number();
    let ldr = LdrImmediateUnsignedOffset::new_mov_from_stack(is_64bit, rd as u8, x.stack_offset)?;

    *pc += 4;
    buf.push(ldr.0.to_le() as i32);

    Ok(())
}

fn encode_mov_from_stack_vector(
    x: &MovFromStack<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    todo!()
}

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/MOV--register---Move--register---an-alias-of-ORR--shifted-register--
fn encode_mov(
    x: &MovOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let source_size = x.source.size();
    let target_size = x.target.size();

    // TODO: Handle Vector Registers
    // a.k.a. `sf` flag
    let is_64bit = if source_size == 8 && target_size == 8 {
        true
    } else if source_size == 16 && target_size == 16 {
        return encode_mov_vector(x, pc, buf);
    } else if source_size == 4 && target_size == 4 {
        false
    } else {
        return Err(JitError::InvalidRegisterCombination(x.source, x.target));
    };

    let rm = x.source.register_number();
    let rd = x.target.register_number();
    let orr = Orr::new_mov(is_64bit, rd as u8, rm as u8);

    *pc += 4;
    buf.push(orr.0.to_le() as i32);

    Ok(())
}

/// # Remarks
///
/// Part of encode_mov, assumes validation already done.
fn encode_mov_vector(
    x: &MovOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    // Note: Validation was already done
    todo!();
    Ok(())
}

macro_rules! encode_branch_relative {
    ($x:expr, $pc:expr, $buf:expr, $opcode:expr) => {{
        // Branch uses number of 4 byte instructions to jump, so we divide by 4.
        // i.e. value of 1 jumps 4 bytes.
        let offset = ($x.target_address as i32 - *$pc as i32) >> 2;

        if !(-0x02000000..=0x01FFFFFF).contains(&offset) {
            return Err(JitError::OperandOutOfRange(
                "Jump distance for Branch Instruction specified too great".to_string(),
            ));
        }

        // Convert the 32-bit offset into a 26-bit offset by shifting right 6 bits
        let imm26 = offset & 0x03FFFFFF;

        // Create the instruction encoding for the B instruction
        let instruction = $opcode << 26 | imm26;

        *$pc += 4;
        $buf.push(instruction.to_le());

        Ok(())
    }};
}

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/BL--Branch-with-Link-
fn encode_call_relative(
    x: &CallRelativeOperation,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    encode_branch_relative!(x, pc, buf, 0b100101)
}

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/B--Branch-
fn encode_jump_relative(
    x: &JumpRelativeOperation,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    encode_branch_relative!(x, pc, buf, 0b000101)
}

#[cfg(test)]

mod tests {
    use core::{mem::size_of_val, slice};

    use crate::all_registers::AllRegisters::*;
    use crate::jit::encode_call_relative;
    use crate::jit::encode_jump_relative;
    use crate::jit::encode_mov;
    use crate::jit::encode_mov_from_stack;
    use crate::jit::encode_pop;
    use crate::jit::encode_push;
    use crate::jit::encode_stackalloc;
    use crate::jit::AllRegisters;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

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
        let operation = JumpRel { target_address };

        assert!(encode_jump_relative(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_pc, pc);
    }

    #[test]
    fn test_encode_jump_relative_invalid_forward() {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = JumpRel {
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
        let operation = JumpRel { target_address };

        assert!(encode_jump_relative(&operation, &mut pc, &mut buf).is_err());
        // Add additional checks for the state of `buf` and `pc` here.
    }

    #[rstest]
    #[case(0, 4, "01000094", 4)] // jump forward
    #[case(4, 0, "ffffff97", 8)] // jump backward
    #[case(4, 4, "00000094", 8)] // no jump
    fn test_encode_call_relative(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
        #[case] expected_pc: usize,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        let operation = CallRel { target_address };

        assert!(encode_call_relative(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_pc, pc);

        // Because this is implemented via macro, skipped other tests for encode_call_relative
    }

    #[rstest]
    #[case(w0, w1, 4, 4, "e003012a")]
    #[case(x0, x1, 8, 8, "e00301aa")]
    #[case(w0, x1, 4, 8, "fail")] // should fail
                                  // #[case(v0, v1, 16, 16, "some_hex_value4")] // vector
    fn test_encode_mov(
        #[case] target: AllRegisters,
        #[case] source: AllRegisters,
        #[case] source_size: usize,
        #[case] target_size: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Mov { source, target };

        // If source and target size don't match, expect an error
        if source_size != target_size {
            assert!(encode_mov(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        assert!(encode_mov(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(4, pc);
    }

    #[rstest]
    #[case(w0, 4, "e00740b9", false)]
    #[case(x0, 8, "e00740f9", false)]
    #[case(w0, 8, "e00b40b9", false)]
    // #[case(AllRegisters::V0, 16, "expected_hex_value3")] // vector, assuming you implement this
    fn test_encode_mov_from_stack(
        #[case] target: AllRegisters,
        #[case] stack_offset: i32,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = MovFromStack {
            stack_offset,
            target,
        };

        // Check the size, expect an error if the size is not 4 or 8
        if is_err {
            assert!(encode_mov_from_stack(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_mov_from_stack(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(4, pc);
    }

    #[rstest]
    #[case(4, "ff1300d1", false)]
    #[case(-4, "ff130091", false)]
    #[case(0, "ff0300d1", false)]
    #[case(2048, "ff0320d1", false)]
    #[case(-2048, "ff032091", false)]
    fn test_encode_stackalloc(
        #[case] operand: i32,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = StackAlloc { operand };

        // Check for errors if applicable
        if is_err {
            assert!(encode_stackalloc(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_stackalloc(&operation, &mut pc, &mut buf).is_ok());

        let encoded_hex = hex::encode(
            buf.iter()
                .flat_map(|&i| i.to_le_bytes().to_vec())
                .collect::<Vec<u8>>(),
        );
        assert_eq!(expected_hex, encoded_hex);

        // Assert that the program counter has been incremented by 4
        assert_eq!(4, pc);
    }

    #[rstest]
    #[case(x0, 4, "e08f1ff8", false)]
    #[case(w0, 4, "e0cf1fb8", false)]
    // #[case(v0, 16, "expected_hex_value_for_vector", false)] // if you implement this
    fn test_encode_push(
        #[case] register: AllRegisters,
        #[case] expected_size: usize,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Push { register };

        // Expect an error for invalid register sizes
        if is_err {
            assert!(encode_push(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_push(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_size, pc);
    }

    #[rstest]
    #[case(x0, 4, "e08740f8", false)]
    #[case(w0, 4, "e04740b8", false)]
    // #[case(v0, 16, "expected_hex_value_for_vector", false)] // if you implement this
    fn test_encode_pop(
        #[case] register: AllRegisters,
        #[case] expected_size: usize,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Pop { register };

        // Expect an error for invalid register sizes
        if is_err {
            assert!(encode_pop(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_pop(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_size, pc);
    }
}
