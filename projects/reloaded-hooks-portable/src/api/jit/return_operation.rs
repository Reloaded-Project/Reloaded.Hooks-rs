use derive_new::new;

/// Represents a return operation which pops the return address from the top of the stack
/// and optionally adjusts the stack pointer by a given offset.
///
/// This can be used to model assembly `ret` instructions with an optional operand.
/// For instance, in x86 assembly, the instruction `ret` would pop the return address
/// from the stack and jump to it. If an operand is provided, such as `ret 4`, it would
/// additionally adjust the stack pointer by the given offset after popping the return address.
///
/// # Fields
///
/// `offset`: This is an optional offset by which the stack pointer is adjusted after
/// popping the return address. If `None`, the stack pointer is not adjusted.
///
/// # Example
///
/// The `ReturnOperation` can be used to represent assembly `ret` instructions.
/// For instance, in x86 assembly, the instruction `ret 4` would pop the return address
/// from the stack, jump to it, and then adjust the stack pointer by 4 bytes. This can be
/// modeled using `ReturnOperation` as follows:
///
/// ```
/// use reloaded_hooks_portable::api::jit::return_operation::ReturnOperation;
/// let ret_4 = ReturnOperation::new(4);
/// ```
///
/// Similarly, a simple `ret` instruction without an operand can be represented as:
///
/// ```
/// use reloaded_hooks_portable::api::jit::return_operation::ReturnOperation;
/// let ret = ReturnOperation::new(0);
/// ```
///
/// # Note
///
/// The `ReturnOperation` only represents the operation itself; it does not perform
/// the operation or modify any actual register or memory values. To simulate the
/// effect of the operation, you would need to implement additional logic or use
/// a computing architecture simulation framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, new)]
pub struct ReturnOperation {
    /// The offset by which the stack pointer is adjusted before returning.
    pub offset: usize,
}
