use derive_new::new;

/// Represents a move operation that moves a register value onto the stack.
///
/// A `MovToStackOperation` consists of a source register to move,
/// and a stack offset to store the value at.
///
/// # Example
///
/// This models assembly instructions that move registers onto the stack.
/// For instance, in Intel syntax:  
///
/// ```
/// use reloaded_hooks_portable::api::jit::mov_to_stack_operation::MovToStackOperation;
///
/// let move_op = MovToStackOperation {
///     register: "eax",
///     stack_offset: 8
/// };
///  
/// // This represents the assembly instruction: MOV [ESP + 8], EAX
/// println!("MOV [ESP + {}], {}", move_op.stack_offset, move_op.register);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, new)]
pub struct MovToStackOperation<T> {
    /// The offset from the current stack pointer in the direction opposite to the stack's growth.
    pub stack_offset: i32,

    /// The source register to move to the stack.
    pub register: T,
}
