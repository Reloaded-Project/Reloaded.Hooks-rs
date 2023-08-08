// JIT for x86
extern crate alloc;

use crate::{
    jit_common::encode_instruction,
    jit_conversions_common::{map_allregisters_to_x86, map_register_x86_to_allregisters},
    x86::register::Register,
};
use alloc::{rc::Rc, string::ToString};
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{
    compiler::{transform_err, Jit, JitError},
    operation::{transform_op, Operation},
};

pub struct JitX86 {}

/// Implementation of the x86 JIT.
impl Jit<Register> for JitX86 {
    fn compile(
        &mut self,
        address: usize,
        operations: &[Operation<Register>],
    ) -> Result<Rc<[u8]>, JitError<Register>> {
        // Initialize Assembler
        let mut a = CodeAssembler::new(32)
            .map_err(|x| JitError::CannotInitializeAssembler(x.to_string()))?;

        // Encode every instruction.
        for operation in operations {
            encoder_instruction_x86(&mut a, operation)?;
        }

        // Assemble those damn instructions
        let result = a
            .assemble(address as u64)
            .map_err(|x| JitError::CannotInitializeAssembler(x.to_string()))?;

        Ok(Rc::from(result))
    }
}

fn encoder_instruction_x86(
    assembler: &mut CodeAssembler,
    operation: &Operation<Register>,
) -> Result<(), JitError<Register>> {
    let all_register_op = transform_op(operation.clone(), |x: Register| {
        map_register_x86_to_allregisters(x)
    });

    encode_instruction(assembler, &all_register_op)
        .map_err(|x| transform_err(x, map_allregisters_to_x86))
}

#[cfg(test)]
mod tests {
    use reloaded_hooks_portable::api::jit::{
        mov_operation::MovOperation, push_operation::PushOperation, sub_operation::SubOperation,
        xchg_operation::XChgOperation,
    };

    use super::*;

    #[test]
    fn test_compile_push() {
        let mut jit = JitX86 {};

        let operations = vec![Operation::Push(PushOperation {
            register: Register::eax,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_mov() {
        let mut jit = JitX86 {};

        let operations = vec![Operation::Mov(MovOperation {
            source: Register::eax,
            target: Register::ebx,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_sub() {
        let mut jit = JitX86 {};

        let operations = vec![Operation::Sub(SubOperation {
            register: Register::eax,
            operand: 10,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_xchg() {
        let mut jit = JitX86 {};

        let operations = vec![Operation::Xchg(XChgOperation {
            register1: Register::eax,
            register2: Register::ebx,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
    }
}
