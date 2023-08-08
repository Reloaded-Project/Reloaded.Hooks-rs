/// Represents a relative call operation to a target address.
///
/// A `CallRelativeOperation` specifies a target address to call with respect to the current instruction's address.
///
/// # Example
///
/// ```
/// use reloaded_hooks_portable::api::jit::call_relative_operation::CallRelativeOperation;
///
/// let call_op = CallRelativeOperation { target_address: 0xDEADBEEF };
///
/// // This might represent an assembly instruction like: CALL offsetOf(0xDEADBEEF) (relative to current instruction address)
/// println!("CALL RelativeOffsetOf(+{:X})", call_op.target_address);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallRelativeOperation {
    /// The target address to call.
    pub target_address: isize,
}

impl CallRelativeOperation {
    /// Creates a new relative call operation.
    pub fn new(target_address: isize) -> Self {
        CallRelativeOperation { target_address }
    }
}
