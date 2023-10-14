/// Represents a relative jump operation to a target address.
///
/// A `JumpRelative` specifies a target address to jump to with respect to the current instruction's address.
///
/// # Example
///
/// ```
/// use reloaded_hooks_portable::api::jit::jump_relative_operation::JumpRelativeOperation;
///
/// let jump_op = JumpRelativeOperation::<i32>::new(0xDEADBEEF);
///
/// // This might represent an assembly instruction like: JMP offsetof(0xDEADBEEF) (relative to current instruction address)
/// println!("JMP RelativeOffsetOf(+{:X})", jump_op.target_address);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpRelativeOperation<T> {
    /// The target address to jump to.
    pub target_address: usize,

    /// Scratch register for the operation
    pub scratch_register: T,
}

impl<T: Default> JumpRelativeOperation<T> {
    /// Creates a new relative jump operation.
    pub fn new(target_address: usize) -> Self {
        JumpRelativeOperation {
            target_address,
            scratch_register: T::default(),
        }
    }
}
