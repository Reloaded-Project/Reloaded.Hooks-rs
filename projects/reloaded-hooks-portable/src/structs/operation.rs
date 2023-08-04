use super::{
    mov_operation::MovOperation, pop_operation::PopOperation, push_operation::PushOperation,
    sub_operation::SubOperation, xchg_operation::XChgOperation,
};

// The operation enum which can represent a Mov, Push, or Sub operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation<T> {
    Mov(MovOperation<T>),
    Push(PushOperation<T>),
    Sub(SubOperation<T>),
    Pop(PopOperation<T>),
    Xchg(XChgOperation<T>),
}
