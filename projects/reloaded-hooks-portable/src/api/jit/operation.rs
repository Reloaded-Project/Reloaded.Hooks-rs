use super::{
    call_operation::CallOperation, jump_absolute_operation::JumpAbsoluteOperation,
    jump_relative_operation::JumpRelativeOperation, mov_operation::MovOperation,
    pop_operation::PopOperation, push_operation::PushOperation,
    push_stack_operation::PushStackOperation, sub_operation::SubOperation,
    xchg_operation::XChgOperation,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation<T> {
    Mov(MovOperation<T>),
    Push(PushOperation<T>),
    PushStack(PushStackOperation<T>),
    Sub(SubOperation<T>),
    Pop(PopOperation<T>),
    Xchg(XChgOperation<T>),
    Call(CallOperation<T>),
    JumpRelative(JumpRelativeOperation),
    JumpAbsolute(JumpAbsoluteOperation<T>),
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
        Operation::Call(inner_op) => Operation::Call(CallOperation {
            relative: inner_op.relative,
            scratch_register: f(inner_op.scratch_register),
            target_address: inner_op.target_address,
        }),
        Operation::JumpRelative(inner_op) => Operation::JumpRelative(JumpRelativeOperation {
            target_address: inner_op.target_address,
        }),
        Operation::JumpAbsolute(inner_op) => Operation::JumpAbsolute(JumpAbsoluteOperation {
            scratch_register: f(inner_op.scratch_register),
            target_address: inner_op.target_address,
        }),
    }
}
