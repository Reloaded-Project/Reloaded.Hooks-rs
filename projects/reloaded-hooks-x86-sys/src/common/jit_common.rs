extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::alloc::string::ToString;
use crate::instructions::{
    call_absolute::encode_call_absolute, call_relative::encode_call_relative,
    jump_absolute::encode_jump_absolute, jump_absolute_indirect::encode_jump_absolute_indirect,
    jump_relative::encode_jump_relative, mov::encode_mov, mov_from_stack::encode_mov_from_stack,
    multi_pop::encode_multi_pop, multi_push::encode_multi_push, pop::encode_pop, push::encode_push,
    push_const::encode_push_constant, push_stack::encode_push_stack, ret::encode_return,
    stack_alloc::encode_stack_alloc, xchg::encode_xchg,
};

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
        Operation::Mov(x) => encode_mov(assembler, x),
        Operation::MovFromStack(x) => encode_mov_from_stack(assembler, x),
        Operation::Push(x) => encode_push(assembler, x),
        Operation::PushStack(x) => encode_push_stack(assembler, x),
        Operation::StackAlloc(x) => encode_stack_alloc(assembler, x),
        Operation::Pop(x) => encode_pop(assembler, x),
        Operation::Xchg(x) => encode_xchg(assembler, x),
        Operation::CallRelative(x) => encode_call_relative(assembler, x),
        Operation::CallAbsolute(x) => encode_call_absolute(assembler, x),
        Operation::JumpRelative(x) => encode_jump_relative(assembler, x),
        Operation::JumpAbsolute(x) => encode_jump_absolute(assembler, x),
        Operation::JumpAbsoluteIndirect(x) => encode_jump_absolute_indirect(assembler, x),

        // x64 only
        #[cfg(feature = "x64")]
        Operation::CallIpRelative(x) => encode_call_ip_relative(assembler, x, address),
        #[cfg(feature = "x64")]
        Operation::JumpIpRelative(x) => encode_jump_ip_relative(assembler, x, address),

        // Optimised Functions
        Operation::MultiPush(x) => encode_multi_push(assembler, x),
        Operation::MultiPop(x) => encode_multi_pop(assembler, x),
        Operation::PushConst(x) => encode_push_constant(assembler, x),
        Operation::Return(x) => encode_return(assembler, x),
        _ => todo!(),
    }
}

pub(crate) fn convert_error(e: IcedError) -> JitError<AllRegisters> {
    JitError::ThirdPartyAssemblerError(e.to_string())
}
