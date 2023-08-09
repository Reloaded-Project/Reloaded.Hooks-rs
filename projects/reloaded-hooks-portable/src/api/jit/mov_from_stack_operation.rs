/// Represents a move operation that moves an element from the stack into a register.
///
/// A `MovFromStackOperation` consists of a stack offset and a target register,
/// representing an instruction to move data from the specified offset on the stack to the target register.
///
/// # Example
///
/// This can be used to model assembly `mov` instructions that transfer data from the stack.
/// For instance, in Intel syntax, a move operation might look like this:
///
/// ```
/// use reloaded_hooks_portable::api::jit::mov_from_stack_operation::MovFromStackOperation;
///
/// let move_op = MovFromStackOperation {
///     stack_offset: 8,
///     target: "eax"
/// };
///
/// // This represents the Intel assembly instruction: MOV EAX, [ESP + 8]
/// println!("MOV {}, [ESP + {}]", move_op.target, move_op.stack_offset);
/// ```
///
/// In the real world, you should use enums instead of strings for the target 😉,
/// the code above shows strings for clarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovFromStackOperation<T> {
    /// The offset from the base of the stack (e.g., ESP or RSP in x86) where the data is located.
    pub stack_offset: i32,

    /// The target (destination) register for the move operation.
    pub target: T,
}
