// Contains simplified types for operations.
// Import as `use crate::api::jit::operation_aliases::*`

use super::{
    call_absolute_operation::CallAbsoluteOperation, call_relative_operation::CallRelativeOperation,
    call_rip_relative_operation::CallIpRelativeOperation,
    jump_absolute_operation::JumpAbsoluteOperation, jump_relative_operation::JumpRelativeOperation,
    jump_rip_relative_operation::JumpIpRelativeOperation,
    mov_from_stack_operation::MovFromStackOperation, mov_operation::MovOperation,
    operation::Operation, pop_operation::PopOperation, push_operation::PushOperation,
    push_stack_operation::PushStackOperation, stack_alloc_operation::StackAllocOperation,
    xchg_operation::XChgOperation,
};

pub type Op<T> = Operation<T>;
pub type Mov<T> = MovOperation<T>;
pub type MovFromStack<T> = MovFromStackOperation<T>;
pub type Push<T> = PushOperation<T>;
pub type PushStack = PushStackOperation;
pub type StackAlloc = StackAllocOperation;
pub type Pop<T> = PopOperation<T>;
pub type XChg<T> = XChgOperation<T>;
pub type CallAbs<T> = CallAbsoluteOperation<T>;
pub type CallRel = CallRelativeOperation;
pub type JumpRel = JumpRelativeOperation;
pub type JumpAbs<T> = JumpAbsoluteOperation<T>;
pub type CallIpRel = CallIpRelativeOperation;
pub type JumpIpRel = JumpIpRelativeOperation;
