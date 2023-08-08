/// Represents a relative jump operation to a target address.
///
/// A `JumpRelative` specifies a target address to jump to with respect to the current instruction's address.
///
/// # Example
///
/// ```
/// use reloaded_hooks_portable::api::jit::jump_operation::JumpRelative;
///
/// let jump_op = JumpRelative { target_offset: 0xDEADBEEF };
///
/// // This might represent an assembly instruction like: JMP 0xDEADBEEF (relative to current instruction address)
/// println!("JMP +{:X}", jump_op.target_offset);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpRelativeOperation {
    /// The target address to jump to.
    pub target_address: usize,
}

impl JumpRelativeOperation {
    /// Creates a new relative jump operation.
    pub fn new(target_address: usize) -> Self {
        JumpRelativeOperation { target_address }
    }
}
