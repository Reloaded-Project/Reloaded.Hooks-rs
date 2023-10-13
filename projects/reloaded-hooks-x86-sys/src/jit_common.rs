extern crate alloc;

use crate::instructions::call_ip_relative::encode_call_ip_relative;
use crate::instructions::call_relative::encode_call_relative;
use crate::instructions::jump_absolute::encode_jump_absolute;
use crate::instructions::jump_ip_relative::encode_jump_ip_relative;
use crate::instructions::jump_relative::encode_jump_relative;
use crate::instructions::mov::encode_mov;
use crate::instructions::mov_from_stack::encode_mov_from_stack;
use crate::instructions::multi_pop::encode_multi_pop;
use crate::instructions::multi_push::encode_multi_push;
use crate::instructions::pop::encode_pop;
use crate::instructions::push::encode_push;
use crate::instructions::push_const::encode_push_constant;
use crate::instructions::push_stack::encode_push_stack;
use crate::instructions::ret::encode_return;
use crate::instructions::stack_alloc::encode_stack_alloc;
use crate::instructions::xchg::encode_xchg;
use crate::instructions::{
    call_absolute::encode_call_absolute, jump_absolute_indirect::encode_jump_absolute_indirect,
};
use crate::jit_common::alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use iced_x86::IcedError;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation::Operation};

use crate::all_registers::AllRegisters;

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
        Operation::CallIpRelative(x) => encode_call_ip_relative(assembler, x, address),
        Operation::JumpIpRelative(x) => encode_jump_ip_relative(assembler, x, address),

        // Optimised Functions
        Operation::MultiPush(x) => encode_multi_push(assembler, x),
        Operation::MultiPop(x) => encode_multi_pop(assembler, x),
        Operation::PushConst(x) => encode_push_constant(assembler, x),
        Operation::Return(x) => encode_return(assembler, x),
        Operation::None => Ok(()),
    }
}

pub(crate) fn convert_error(e: IcedError) -> JitError<AllRegisters> {
    JitError::ThirdPartyAssemblerError(e.to_string())
}
