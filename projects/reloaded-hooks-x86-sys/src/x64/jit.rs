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
    compiler::{transform_err, Jit, JitCapabilities, JitError},
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

    fn max_relative_jump_distance() -> usize {
        i32::MAX as usize
    }

    fn get_jit_capabilities() -> &'static [JitCapabilities] {
        &[
            JitCapabilities::CanEncodeIPRelativeCall,
            JitCapabilities::CanEncodeIPRelativeJump,
            JitCapabilities::CanMultiPush,
        ]
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

#[cfg(test)]
mod tests {
    use reloaded_hooks_portable::api::jit::{
        call_absolute_operation::CallAbsoluteOperation,
        call_relative_operation::CallRelativeOperation,
        call_rip_relative_operation::CallIpRelativeOperation,
        jump_absolute_operation::JumpAbsoluteOperation,
        jump_relative_operation::JumpRelativeOperation,
        jump_rip_relative_operation::JumpIpRelativeOperation,
        mov_from_stack_operation::MovFromStackOperation, mov_operation::MovOperation,
        pop_operation::PopOperation, push_operation::PushOperation, sub_operation::SubOperation,
        xchg_operation::XChgOperation,
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
        assert_eq!("50", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_push_xmm() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::Push(PushOperation {
            register: Register::xmm0,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("4883ec10f30f7f0424", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_push_ymm() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::Push(PushOperation {
            register: Register::ymm0,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("4883ec20c5fe7f0424", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_push_zmm() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::Push(PushOperation {
            register: Register::zmm0,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(
            "4883ec4062f17f487f0424",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn test_compile_pop_xmm() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::Pop(PopOperation {
            register: Register::xmm0,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("f30f6f04244883c410", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_pop_ymm() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::Pop(PopOperation {
            register: Register::ymm0,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("c5fe6f04244883c420", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_pop_zmm() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::Pop(PopOperation {
            register: Register::zmm0,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("62f17f486f04244883c440", hex::encode(result.unwrap()));
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
        assert_eq!("4889c3", hex::encode(result.unwrap()));
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
        assert_eq!("482d0a000000", hex::encode(result.unwrap()));
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
        assert_eq!("4887d8", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_jmp_relative() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::JumpRelative(JumpRelativeOperation {
            target_address: 0x7FFFFFFF,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("e9faffff7f", hex::encode(result.unwrap()));
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
        assert_eq!(
            "48b87856341200000000ffe0",
            hex::encode(result.as_ref().unwrap())
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
        assert_eq!("e8faffff7f", hex::encode(result.unwrap()));
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
        assert_eq!("48b87856341200000000ffd0", hex::encode(result.unwrap()));
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
    fn test_compile_jmp_relative_is_relative_to_rip() {
        let mut jit = JitX64 {};

        // Verifies that the JIT compiles a relative jmp that branches towards target_address
        // This is verified by branching to an address outside of the 2GB range and setting
        // Instruction Pointer of assembled code to make it within range.
        let operations = vec![Operation::JumpRelative(JumpRelativeOperation {
            target_address: 0x80000005,
        })];
        let result = jit.compile(5, &operations);
        assert!(result.is_ok());
        assert_eq!("e9fbffff7f", hex::encode(result.as_ref().unwrap()));
    }

    #[test]
    #[should_panic]
    fn test_compile_call_relative_out_of_range() {
        let mut jit = JitX64 {};

        // Note: This fails inside Iced :/
        let operations = vec![Operation::CallRelative(CallRelativeOperation {
            target_address: usize::MAX,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_call_relative_is_relative_to_rip() {
        let mut jit = JitX64 {};

        // Verifies that the JIT compiles a relative call that branches towards target_address
        // This is verified by branching to an address outside of the 2GB range and setting
        // Instruction Pointer of assembled code to make it within range.
        let operations = vec![Operation::CallRelative(CallRelativeOperation {
            target_address: 0x80000005,
        })];
        let result = jit.compile(5, &operations);
        assert!(result.is_ok());
        assert_eq!("e8fbffff7f", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_call_rip_relative() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::CallIpRelative(CallIpRelativeOperation {
            target_address: 0x16,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("ff1510000000", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_jmp_rip_relative() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::JumpIpRelative(JumpIpRelativeOperation {
            target_address: 0x16,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("ff2510000000", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_call_rip_relative_backwards() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::CallIpRelative(CallIpRelativeOperation {
            target_address: 16,
        })];
        let result = jit.compile(20, &operations);
        assert!(result.is_ok());
        assert_eq!("ff15e2ffffff", hex::encode(result.as_ref().unwrap()))
    }

    #[test]
    fn test_compile_jmp_rip_relative_backwards() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::JumpIpRelative(JumpIpRelativeOperation {
            target_address: 16,
        })];
        let result = jit.compile(20, &operations);
        assert!(result.is_ok());
        assert_eq!("ff25e2ffffff", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_call_rip_relative_two_instructions() {
        let mut jit = JitX64 {};

        let operations = vec![
            Operation::Sub(SubOperation {
                register: Register::rax,
                operand: 10,
            }),
            Operation::CallIpRelative(CallIpRelativeOperation { target_address: 16 }),
        ];
        let result = jit.compile(20, &operations);
        assert!(result.is_ok());
        assert_eq!("482d0a000000ff15f0ffffff", hex::encode(result.unwrap()));
    }

    #[test]
    fn test_compile_jmp_rip_relative_backwards_two_instructions() {
        let mut jit = JitX64 {};

        let operations = vec![
            Operation::Sub(SubOperation {
                register: Register::rax,
                operand: 10,
            }),
            Operation::JumpIpRelative(JumpIpRelativeOperation { target_address: 16 }),
        ];
        let result = jit.compile(20, &operations);
        assert!(result.is_ok());
        assert_eq!(
            "482d0a000000ff25f0ffffff",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn test_compile_mov_from_stack() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MovFromStack(MovFromStackOperation {
            stack_offset: 4,
            target: Register::rax,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("488b442404", hex::encode(result.as_ref().unwrap()));
    }

    #[test]
    fn compile_multi_push_basic_regs_only() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MultiPush(vec![
            PushOperation {
                register: Register::rax,
            },
            PushOperation {
                register: Register::rbx,
            },
            PushOperation {
                register: Register::rcx,
            },
        ])];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());

        // The expected output is a placeholder. You'll need to replace it with the correct hex string.
        assert_eq!(
            "83ec1848890c2448895c24084889442410",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn compile_multi_push_xmm_only() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MultiPush(vec![
            PushOperation {
                register: Register::xmm0,
            },
            PushOperation {
                register: Register::xmm1,
            },
            PushOperation {
                register: Register::xmm2,
            },
        ])];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());

        // The expected output is a placeholder. You'll need to replace it with the correct hex string.
        assert_eq!(
            "83ec30f30f7f1424f30f7f4c2410f30f7f442420",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn compile_multi_push_ymm_only() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MultiPush(vec![
            PushOperation {
                register: Register::ymm0,
            },
            PushOperation {
                register: Register::ymm1,
            },
            PushOperation {
                register: Register::ymm2,
            },
        ])];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());

        // The expected output is a placeholder. You'll need to replace it with the correct hex string.
        assert_eq!(
            "83ec60c5fe7f1424c5fe7f4c2420c5fe7f442440",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn compile_multi_push_zmm_only() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MultiPush(vec![
            PushOperation {
                register: Register::zmm0,
            },
            PushOperation {
                register: Register::zmm1,
            },
            PushOperation {
                register: Register::zmm2,
            },
        ])];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());

        // The expected output is a placeholder. You'll need to replace it with the correct hex string.
        assert_eq!(
            "81ecc000000062f17f487f142462f17f487f4c240162f17f487f442402",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn compile_multi_push_mixed() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MultiPush(vec![
            PushOperation {
                register: Register::rax,
            },
            PushOperation {
                register: Register::xmm0,
            },
            PushOperation {
                register: Register::ymm1,
            },
        ])];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());

        // The expected output is a placeholder. You'll need to replace it with the correct hex string.
        assert_eq!(
            "83ec38c5fe7f0c24f30f7f4424204889442430",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn compile_multi_pop_basic_regs_only() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MultiPop(vec![
            PopOperation {
                register: Register::rax,
            },
            PopOperation {
                register: Register::rbx,
            },
            PopOperation {
                register: Register::rcx,
            },
        ])];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());

        // The expected output is a placeholder. You'll need to replace it with the correct hex string.
        assert_eq!(
            "488b0424488b5c2408488b4c241083c418",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn compile_multi_pop_xmm_only() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MultiPop(vec![
            PopOperation {
                register: Register::xmm0,
            },
            PopOperation {
                register: Register::xmm1,
            },
            PopOperation {
                register: Register::xmm2,
            },
        ])];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());

        // The expected output is a placeholder. You'll need to replace it with the correct hex string.
        assert_eq!(
            "f30f6f0424f30f6f4c2410f30f6f54242083c430",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn compile_multi_pop_ymm_only() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MultiPop(vec![
            PopOperation {
                register: Register::ymm0,
            },
            PopOperation {
                register: Register::ymm1,
            },
            PopOperation {
                register: Register::ymm2,
            },
        ])];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());

        // The expected output is a placeholder. You'll need to replace it with the correct hex string.
        assert_eq!(
            "c5fe6f0424c5fe6f4c2420c5fe6f54244083c460",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn compile_multi_pop_zmm_only() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MultiPop(vec![
            PopOperation {
                register: Register::zmm0,
            },
            PopOperation {
                register: Register::zmm1,
            },
            PopOperation {
                register: Register::zmm2,
            },
        ])];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());

        // The expected output is a placeholder. You'll need to replace it with the correct hex string.
        assert_eq!(
            "62f17f486f042462f17f486f4c240162f17f486f54240281c4c0000000",
            hex::encode(result.as_ref().unwrap())
        );
    }

    #[test]
    fn compile_multi_pop_mixed() {
        let mut jit = JitX64 {};

        let operations = vec![Operation::MultiPop(vec![
            PopOperation {
                register: Register::rax,
            },
            PopOperation {
                register: Register::xmm0,
            },
            PopOperation {
                register: Register::ymm1,
            },
        ])];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());

        // The expected output is a placeholder. You'll need to replace it with the correct hex string.
        assert_eq!(
            "488b0424f30f6f442408c5fe6f4c241883c438",
            hex::encode(result.as_ref().unwrap())
        );
    }
}
