// JIT for x64
extern crate alloc;

use crate::{
    jit_common::encode_instruction,
    jit_conversions_common::{map_allregisters_to_x64, map_register_x64_to_allregisters},
    x64::register::Register,
};
use alloc::{rc::Rc, string::ToString};
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{
    compiler::{transform_err, Jit, JitError},
    operation::{transform_op, Operation},
};

pub struct JitX64 {}

/// Implementation of the x64 JIT.
impl Jit<Register> for JitX64 {
    fn compile(
        &mut self,
        address: usize,
        operations: &[Operation<Register>],
    ) -> Result<Rc<[u8]>, JitError<Register>> {
        // Initialize Assembler
        let mut a = CodeAssembler::new(64)
            .map_err(|x| JitError::CannotInitializeAssembler(x.to_string()))?;

        // Encode every instruction.
        for operation in operations {
            encoder_instruction_x64(&mut a, operation)?;
        }

        // Assemble those damn instructions
        let result = a
            .assemble(address as u64)
            .map_err(|x| JitError::CannotInitializeAssembler(x.to_string()))?;

        Ok(Rc::from(result))
    }
}

fn encoder_instruction_x64(
    assembler: &mut CodeAssembler,
    operation: &Operation<Register>,
) -> Result<(), JitError<Register>> {
    let all_register_op = transform_op(operation.clone(), |x: Register| {
        map_register_x64_to_allregisters(x)
    });

    encode_instruction(assembler, &all_register_op)
        .map_err(|x| transform_err(x, map_allregisters_to_x64))
}

#[cfg(test)]
mod tests {
    use reloaded_hooks_portable::api::jit::{
        call_absolute_operation::CallAbsoluteOperation,
        call_relative_operation::CallRelativeOperation,
        jump_absolute_operation::JumpAbsoluteOperation,
        jump_relative_operation::JumpRelativeOperation, mov_operation::MovOperation,
        push_operation::PushOperation, sub_operation::SubOperation, xchg_operation::XChgOperation,
    };

    use super::*;

    // run as `cargo test -- --nocapture` for printing

    #[test]
    fn test_compile_push() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::Push(PushOperation {
            register: Register::rax,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        println!("x64::test_compile_push: {}", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_mov() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::Mov(MovOperation {
            source: Register::rax,
            target: Register::rbx,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        println!("x64::test_compile_mov: {}", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_sub() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::Sub(SubOperation {
            register: Register::rax,
            operand: 10,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        println!("x64::test_compile_sub: {}", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_xchg() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::Xchg(XChgOperation {
            register1: Register::rax,
            register2: Register::rbx,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        println!("x64::test_compile_xchg: {}", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_jmp_relative() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::JumpRelative(JumpRelativeOperation {
            target_address: 0x7FFFFFFF,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        println!(
            "x64::test_compile_jmp_relative: {}",
            hex::encode(result.unwrap())
        );
    }

    #[test]
    fn test_compile_jmp_absolute() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::JumpAbsolute(JumpAbsoluteOperation {
            scratch_register: Register::rax,
            target_address: 0x12345678,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        println!(
            "x64::test_compile_jmp_absolute: {}",
            hex::encode(result.unwrap())
        );
    }

    #[test]
    fn test_compile_call_relative() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::CallRelative(CallRelativeOperation {
            target_address: 0x7FFFFFFF,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        println!(
            "x64::test_compile_call_relative: {}",
            hex::encode(result.unwrap())
        );
    }

    #[test]
    fn test_compile_call_absolute() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::CallAbsolute(CallAbsoluteOperation {
            scratch_register: Register::rax,
            target_address: 0x12345678,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        println!(
            "x64::test_compile_call_absolute: {}",
            hex::encode(result.unwrap())
        );
    }

    #[test]
    #[should_panic]
    fn test_compile_jmp_relative_out_of_range() {
        let mut jit = JitX64 {};

        // Note: This fails inside Iced :/
        let operations = vec![Operation::JumpRelative(JumpRelativeOperation {
            target_address: usize::MAX,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_jmp_relative_is_relative_to_eip() {
        let mut jit = JitX64 {};

        // Verifies that the JIT compiles a relative jmp that branches towards target_address
        // This is verified by branching to an address outside of the 2GB range and setting
        // Instruction Pointer of assembled code to make it within range.
        let operations = vec![Operation::JumpRelative(JumpRelativeOperation {
            target_address: 0x80000005,
        })];
        let result = jit.compile(5, &operations);
        assert!(result.is_ok());
        println!(
            "x64::test_compile_call_relative_is_relative_to_eip: {}",
            hex::encode(result.unwrap())
        );
    }

    #[test]
    #[should_panic]
    fn test_compile_call_relative_out_of_range() {
        let mut jit = JitX64 {};

        // Note: This fails inside Iced :/
        let operations = vec![Operation::JumpRelative(JumpRelativeOperation {
            target_address: usize::MAX,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_call_relative_is_relative_to_eip() {
        let mut jit = JitX64 {};

        // Verifies that the JIT compiles a relative call that branches towards target_address
        // This is verified by branching to an address outside of the 2GB range and setting
        // Instruction Pointer of assembled code to make it within range.
        let operations = vec![Operation::JumpRelative(JumpRelativeOperation {
            target_address: 0x80000005,
        })];
        let result = jit.compile(5, &operations);
        assert!(result.is_ok());
        println!(
            "x64::test_compile_call_relative_is_relative_to_eip: {}",
            hex::encode(result.unwrap())
        );
    }
}
