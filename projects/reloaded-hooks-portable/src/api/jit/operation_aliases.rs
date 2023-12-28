// Contains simplified types for operations.
// Import as `use crate::api::jit::operation_aliases::*`

use super::{
    call_absolute_operation::CallAbsoluteOperation, call_relative_operation::CallRelativeOperation,
    call_rip_relative_operation::CallIpRelativeOperation,
    jump_absolute_indirect_operation::JumpAbsoluteIndirectOperation,
    jump_absolute_operation::JumpAbsoluteOperation, jump_relative_operation::JumpRelativeOperation,
    jump_rip_relative_operation::JumpIpRelativeOperation,
    mov_from_stack_operation::MovFromStackOperation, mov_operation::MovOperation,
    mov_to_stack_operation::MovToStackOperation, operation::Operation, pop_operation::PopOperation,
    push_constant_operation::PushConstantOperation, push_operation::PushOperation,
    push_stack_operation::PushStackOperation, return_operation::ReturnOperation,
    stack_alloc_operation::StackAllocOperation, xchg_operation::XChgOperation,
};

pub type Op<T> = Operation<T>;
pub type Mov<T> = MovOperation<T>;
pub type MovFromStack<T> = MovFromStackOperation<T>;
pub type Push<T> = PushOperation<T>;
pub type PushConst<T> = PushConstantOperation<T>;
pub type PushStack<T> = PushStackOperation<T>;
pub type StackAlloc = StackAllocOperation;
pub type Pop<T> = PopOperation<T>;
pub type XChg<T> = XChgOperation<T>;
pub type CallAbs<T> = CallAbsoluteOperation<T>;
pub type CallRel = CallRelativeOperation;
pub type JumpRel<T> = JumpRelativeOperation<T>;
pub type JumpAbs<T> = JumpAbsoluteOperation<T>;
pub type JumpAbsInd<T> = JumpAbsoluteIndirectOperation<T>;
pub type CallIpRel<T> = CallIpRelativeOperation<T>;
pub type JumpIpRel<T> = JumpIpRelativeOperation<T>;
pub type MovToStack<T> = MovToStackOperation<T>;
pub type Return = ReturnOperation;
