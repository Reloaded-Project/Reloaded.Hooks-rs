/// Represents a move operation between two registers.
///
/// A `MovOperation` consists of a source and a target register,
/// representing an instruction to move data from the source to the target.
///
/// # Example
///
/// This can be used to model assembly `mov` instructions.
/// For instance, in Intel syntax, a move operation might look like this:
///
/// ```
/// use reloaded_hooks_portable::api::jit::mov_operation::MovOperation;
///
/// let move_op = MovOperation {
///     source: "eax",
///     target: "ebx"
/// };
///
/// // This represents the Intel assembly instruction: MOV EBX, EAX
/// println!("MOV {}, {}", move_op.target, move_op.source);
/// ```
///
/// In the real world, you should use enums instead of strings for source and target 😉,
/// the code above shows strings for clarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovOperation<T> {
    /// The source register for the move operation.
    pub source: T,

    /// The target (destination) register for the move operation.
    pub target: T,
}
