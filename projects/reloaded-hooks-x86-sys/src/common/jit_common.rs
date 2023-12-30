extern crate alloc;
use crate::all_registers::AllRegisters;
use crate::instructions::{
    call_absolute::encode_call_absolute, call_relative::encode_call_relative,
    jump_absolute::encode_jump_absolute, jump_absolute_indirect::encode_jump_absolute_indirect,
    jump_relative::encode_jump_relative, mov::encode_mov, mov_from_stack::encode_mov_from_stack,
    mov_to_stack::encode_mov_to_stack, pop::encode_pop, push::encode_push,
    push_const::encode_push_constant, push_stack::encode_push_stack, ret::encode_return,
    stack_alloc::encode_stack_alloc, xchg::encode_xchg,
};
use alloc::string::ToString;

#[cfg(target_feature = "multipushpop")]
use crate::instructions::multi_pop::encode_multi_pop;

#[cfg(target_feature = "multipushpop")]
use crate::instructions::multi_push::encode_multi_push;

#[cfg(feature = "x64")]
use crate::instructions::jump_ip_relative::encode_jump_ip_relative;

#[cfg(feature = "x64")]
use crate::instructions::call_ip_relative::encode_call_ip_relative;

use iced_x86::{code_asm::CodeAssembler, IcedError};
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation::Operation};

pub const ARCH_NOT_SUPPORTED: &str = "Non 32/64bit architectures are not supported";

pub(crate) fn encode_instruction(
    assembler: &mut CodeAssembler,
    operation: &Operation<AllRegisters>,
    address: usize,
) -> Result<(), JitError<AllRegisters>> {
    match operation {
        Operation::Mov(x) => Ok(encode_mov(assembler, x)?),
        Operation::MovFromStack(x) => Ok(encode_mov_from_stack(assembler, x)?),
        Operation::Push(x) => Ok(encode_push(assembler, x)?),
        Operation::PushStack(x) => Ok(encode_push_stack(assembler, x)?),
        Operation::StackAlloc(x) => Ok(encode_stack_alloc(assembler, x)?),
        Operation::Pop(x) => Ok(encode_pop(assembler, x)?),
        Operation::Xchg(x) => Ok(encode_xchg(assembler, x)?),
        Operation::CallRelative(x) => Ok(encode_call_relative(assembler, x)?),
        Operation::CallAbsolute(x) => Ok(encode_call_absolute(assembler, x)?),
        Operation::JumpRelative(x) => Ok(encode_jump_relative(assembler, x)?),
        Operation::JumpAbsolute(x) => Ok(encode_jump_absolute(assembler, x)?),
        Operation::JumpAbsoluteIndirect(x) => Ok(encode_jump_absolute_indirect(assembler, x)?),
        Operation::MovToStack(x) => Ok(encode_mov_to_stack(assembler, x)?),

        // x64 only
        #[cfg(feature = "x64")]
        Operation::CallIpRelative(x) => Ok(encode_call_ip_relative(assembler, x, address)?),
        #[cfg(feature = "x64")]
        Operation::JumpIpRelative(x) => Ok(encode_jump_ip_relative(assembler, x, address)?),

        // Optimised Functions
        Operation::PushConst(x) => Ok(encode_push_constant(assembler, x)?),
        Operation::Return(x) => Ok(encode_return(assembler, x)?),

        // Deprecated
        #[cfg(target_feature = "multipushpop")]
        Operation::MultiPush(x) => encode_multi_push(assembler, x),

        #[cfg(target_feature = "multipushpop")]
        Operation::MultiPop(x) => encode_multi_pop(assembler, x),
        _ => unreachable!(),
    }
}

pub enum X86jitError<T> {
    IcedError(IcedError),
    JitError(JitError<T>),
}

impl<T> From<IcedError> for X86jitError<T> {
    fn from(e: IcedError) -> Self {
        Self::IcedError(e)
    }
}

impl<T> From<JitError<T>> for X86jitError<T> {
    fn from(e: JitError<T>) -> Self {
        Self::JitError(e)
    }
}

impl<T> From<X86jitError<T>> for JitError<T> {
    fn from(val: X86jitError<T>) -> Self {
        match val {
            X86jitError::JitError(e) => e,
            X86jitError::IcedError(e) => JitError::ThirdPartyAssemblerError(e.to_string()),
        }
    }
}
