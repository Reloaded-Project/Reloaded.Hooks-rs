extern crate alloc;
use super::{
    call_absolute_operation::CallAbsoluteOperation, call_relative_operation::CallRelativeOperation,
    call_rip_relative_operation::CallIpRelativeOperation,
    jump_absolute_indirect_operation::JumpAbsoluteIndirectOperation,
    jump_absolute_operation::JumpAbsoluteOperation, jump_relative_operation::JumpRelativeOperation,
    jump_rip_relative_operation::JumpIpRelativeOperation,
    mov_from_stack_operation::MovFromStackOperation, mov_operation::MovOperation,
    mov_to_stack_operation::MovToStackOperation, pop_operation::PopOperation,
    push_constant_operation::PushConstantOperation, push_operation::PushOperation,
    push_stack_operation::PushStackOperation, return_operation::ReturnOperation,
    stack_alloc_operation::StackAllocOperation, xchg_operation::XChgOperation,
};
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use derive_more::From;
use smallvec::SmallVec;

pub type MultiPushVec<T> = [PushOperation<T>; 4];
pub type MultiPopVec<T> = [PopOperation<T>; 4];

#[derive(Debug, Clone, PartialEq, Eq, From)]
pub enum Operation<T: Copy + Clone> {
    None,
    Mov(MovOperation<T>),
    MovFromStack(MovFromStackOperation<T>),
    Push(PushOperation<T>),
    PushStack(PushStackOperation<T>),
    PushConst(PushConstantOperation<T>), // Required for parameter injection
    StackAlloc(StackAllocOperation),
    Pop(PopOperation<T>),
    Xchg(XChgOperation<T>),
    CallAbsolute(CallAbsoluteOperation<T>),
    CallRelative(CallRelativeOperation),
    JumpRelative(JumpRelativeOperation<T>),
    JumpAbsolute(JumpAbsoluteOperation<T>),
    JumpAbsoluteIndirect(JumpAbsoluteIndirectOperation<T>),
    Return(ReturnOperation),

    // Only possible on some architectures.
    // These are opt-in and controlled by [JitCapabilities](super::compiler::JitCapabilities).
    CallIpRelative(CallIpRelativeOperation<T>),
    JumpIpRelative(JumpIpRelativeOperation<T>),
    MovToStack(MovToStackOperation<T>),
    // Opt-in for architectures that support it or can optimise for this use case.
    // These are opt-in and controlled by [JitCapabilities](super::compiler::JitCapabilities).

    // Note: I experimented with packing, to try make push/pull 1 byte, but seemed to have no effect.
    MultiPush(SmallVec<MultiPushVec<T>>),
    MultiPop(SmallVec<MultiPopVec<T>>),
}

pub fn transform_op<TOldRegister: Copy + Clone, TNewRegister: Copy + Clone, TConvertRegister>(
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
        Operation::PushStack(inner_op) => {
            // TODO: This is slow, due to a full copy. However this is only hit in presence of external assemblers.
            let borrowed_scratch = inner_op.scratch.borrow();
            let mut new_vec = Vec::with_capacity(borrowed_scratch.len());
            new_vec.extend(borrowed_scratch.iter().map(|x| f(*x)));
            Operation::PushStack(PushStackOperation {
                offset: inner_op.offset,
                item_size: inner_op.item_size,
                scratch: Rc::new(RefCell::new(new_vec)),
            })
        }
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

        Operation::JumpRelative(inner_op) => Operation::JumpRelative(JumpRelativeOperation {
            target_address: inner_op.target_address,
            scratch_register: f(inner_op.scratch_register),
        }),
        Operation::JumpAbsolute(inner_op) => Operation::JumpAbsolute(JumpAbsoluteOperation {
            scratch_register: f(inner_op.scratch_register),
            target_address: inner_op.target_address,
        }),
        Operation::CallIpRelative(inner_op) => Operation::CallIpRelative(CallIpRelativeOperation {
            scratch: f(inner_op.scratch),
            target_address: inner_op.target_address,
        }),
        Operation::JumpIpRelative(inner_op) => Operation::JumpIpRelative(JumpIpRelativeOperation {
            scratch: f(inner_op.scratch),
            target_address: inner_op.target_address,
        }),
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
        Operation::PushConst(x) => Operation::PushConst(PushConstantOperation {
            value: x.value,
            scratch: x.scratch.map(f),
        }),
        Operation::Return(x) => Operation::Return(x),
        Operation::None => Operation::None,
        Operation::JumpAbsoluteIndirect(inner_op) => {
            Operation::JumpAbsoluteIndirect(JumpAbsoluteIndirectOperation {
                scratch_register: inner_op.scratch_register.map(f),
                pointer_address: inner_op.pointer_address,
            })
        }
        Operation::MovToStack(x) => Operation::MovToStack(MovToStackOperation {
            register: f(x.register),
            stack_offset: x.stack_offset,
        }),
    }
}
