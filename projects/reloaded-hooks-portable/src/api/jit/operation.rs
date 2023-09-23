extern crate alloc;

use derive_more::From;
use smallvec::SmallVec;

use super::{
    call_absolute_operation::CallAbsoluteOperation, call_relative_operation::CallRelativeOperation,
    call_rip_relative_operation::CallIpRelativeOperation,
    jump_absolute_operation::JumpAbsoluteOperation, jump_relative_operation::JumpRelativeOperation,
    jump_rip_relative_operation::JumpIpRelativeOperation,
    mov_from_stack_operation::MovFromStackOperation, mov_operation::MovOperation,
    pop_operation::PopOperation, push_constant_operation::PushConstantOperation,
    push_operation::PushOperation, push_stack_operation::PushStackOperation,
    return_operation::ReturnOperation, stack_alloc_operation::StackAllocOperation,
    xchg_operation::XChgOperation,
};

pub type MultiPushVec<T> = [PushOperation<T>; 4];
pub type MultiPopVec<T> = [PopOperation<T>; 4];

#[derive(Debug, Clone, PartialEq, Eq, From)]
pub enum Operation<T> {
    None,
    Mov(MovOperation<T>),
    MovFromStack(MovFromStackOperation<T>),
    Push(PushOperation<T>),
    PushStack(PushStackOperation<T>),
    PushConst(PushConstantOperation), // Required for parameter injection
    StackAlloc(StackAllocOperation),
    Pop(PopOperation<T>),
    Xchg(XChgOperation<T>),
    CallAbsolute(CallAbsoluteOperation<T>),
    CallRelative(CallRelativeOperation),
    JumpRelative(JumpRelativeOperation),
    JumpAbsolute(JumpAbsoluteOperation<T>),
    Return(ReturnOperation),

    // Only possible on some architectures.
    // These are opt-in and controlled by [JitCapabilities](super::compiler::JitCapabilities).
    CallIpRelative(CallIpRelativeOperation),
    JumpIpRelative(JumpIpRelativeOperation),

    // Opt-in for architectures that support it or can optimise for this use case.
    // These are opt-in and controlled by [JitCapabilities](super::compiler::JitCapabilities).

    // Note: I experimented with packing, to try make push/pull 1 byte, but seemed to have no effect.
    MultiPush(SmallVec<MultiPushVec<T>>),
    MultiPop(SmallVec<MultiPopVec<T>>),
}

pub fn transform_op<TOldRegister: Copy, TNewRegister, TConvertRegister>(
    op: Operation<TOldRegister>,
    f: TConvertRegister,
) -> Operation<TNewRegister>
where
    TConvertRegister: Fn(TOldRegister) -> TNewRegister,
{
    match op {
        Operation::Mov(inner_op) => Operation::Mov(MovOperation {
            source: f(inner_op.source),
            target: f(inner_op.target),
        }),
        Operation::Push(inner_op) => Operation::Push(PushOperation {
            register: f(inner_op.register),
        }),
        Operation::PushStack(inner_op) => Operation::PushStack(PushStackOperation {
            offset: inner_op.offset,
            item_size: inner_op.item_size,
            scratch_1: inner_op.scratch_1.map(&f),
            scratch_2: inner_op.scratch_2.map(&f),
        }),
        Operation::StackAlloc(inner_op) => Operation::StackAlloc(StackAllocOperation {
            operand: inner_op.operand,
        }),
        Operation::Pop(inner_op) => Operation::Pop(PopOperation {
            register: f(inner_op.register),
        }),
        Operation::Xchg(inner_op) => Operation::Xchg(XChgOperation {
            register1: f(inner_op.register1),
            register2: f(inner_op.register2),
            scratch: inner_op.scratch.map(&f),
        }),
        Operation::CallRelative(inner_op) => Operation::CallRelative(inner_op),
        Operation::CallAbsolute(inner_op) => Operation::CallAbsolute(CallAbsoluteOperation {
            scratch_register: f(inner_op.scratch_register),
            target_address: inner_op.target_address,
        }),

        Operation::JumpRelative(inner_op) => Operation::JumpRelative(inner_op),
        Operation::JumpAbsolute(inner_op) => Operation::JumpAbsolute(JumpAbsoluteOperation {
            scratch_register: f(inner_op.scratch_register),
            target_address: inner_op.target_address,
        }),
        Operation::CallIpRelative(inner_op) => Operation::CallIpRelative(inner_op),
        Operation::JumpIpRelative(inner_op) => Operation::JumpIpRelative(inner_op),
        Operation::MovFromStack(inner_op) => Operation::MovFromStack(MovFromStackOperation {
            stack_offset: inner_op.stack_offset,
            target: f(inner_op.target),
        }),
        Operation::MultiPush(inner_ops) => Operation::MultiPush(
            inner_ops
                .iter()
                .map(|op| PushOperation {
                    register: f(op.register),
                })
                .collect(),
        ),
        Operation::MultiPop(inner_ops) => Operation::MultiPop(
            inner_ops
                .iter()
                .map(|op| PopOperation {
                    register: f(op.register),
                })
                .collect(),
        ),
        Operation::PushConst(x) => Operation::PushConst(x),
        Operation::Return(x) => Operation::Return(x),
        Operation::None => Operation::None,
    }
}
