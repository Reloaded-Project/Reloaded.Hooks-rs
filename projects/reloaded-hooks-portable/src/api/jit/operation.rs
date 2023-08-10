use super::{
    call_absolute_operation::CallAbsoluteOperation, call_relative_operation::CallRelativeOperation,
    call_rip_relative_operation::CallIpRelativeOperation,
    jump_absolute_operation::JumpAbsoluteOperation, jump_relative_operation::JumpRelativeOperation,
    jump_rip_relative_operation::JumpIpRelativeOperation,
    mov_from_stack_operation::MovFromStackOperation, mov_operation::MovOperation,
    pop_operation::PopOperation, push_operation::PushOperation,
    push_stack_operation::PushStackOperation, sub_operation::SubOperation,
    xchg_operation::XChgOperation,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Operation<T> {
    Mov(MovOperation<T>),
    MovFromStack(MovFromStackOperation<T>),
    Push(PushOperation<T>),
    PushStack(PushStackOperation<T>),
    Sub(SubOperation<T>),
    Pop(PopOperation<T>),
    Xchg(XChgOperation<T>),
    CallAbsolute(CallAbsoluteOperation<T>),
    CallRelative(CallRelativeOperation),
    JumpRelative(JumpRelativeOperation),
    JumpAbsolute(JumpAbsoluteOperation<T>),

    // Only possible on some architectures.
    CallIpRelative(CallIpRelativeOperation),
    JumpIpRelative(JumpIpRelativeOperation),
}

pub fn transform_op<TOldRegister, TNewRegister, TConvertRegister>(
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
            base_register: f(inner_op.base_register),
            offset: inner_op.offset,
            item_size: inner_op.item_size,
        }),
        Operation::Sub(inner_op) => Operation::Sub(SubOperation {
            register: f(inner_op.register),
            operand: inner_op.operand,
        }),
        Operation::Pop(inner_op) => Operation::Pop(PopOperation {
            register: f(inner_op.register),
        }),
        Operation::Xchg(inner_op) => Operation::Xchg(XChgOperation {
            register1: f(inner_op.register1),
            register2: f(inner_op.register2),

            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
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
    }
}
