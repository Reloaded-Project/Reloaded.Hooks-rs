/// Represents an absolute call operation using a specific register and target address.
///
/// A `CallAbsoluteOperation` specifies which register to use as a base and the target address to call.
///
/// # Example
///
/// ```
/// use reloaded_hooks_portable::api::jit::call_absolute_operation::CallAbsoluteOperation;
///
/// let call_op = CallAbsoluteOperation {
///     scratch_register: "rax",
///     target_address: 0xDEADBEEF,
/// };
///
/// // This might represent an assembly instruction sequence like:
/// //  MOV RAX, 0xDEADBEEF
/// //  CALL RAX
///
/// // Or if scratch_register is not specified, it will represent
/// // call qword [0x123456] // at address 0x123456 is 0xDEADBEEF
/// ```
///
/// In a real-world scenario, you'd likely use enums instead of strings for the scratch register.
/// The code above uses a string for demonstration purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallAbsoluteOperation<T> {
    /// The scratch register to use for the call operation.
    pub scratch_register: T,

    /// The target address to call.
    pub target_address: usize,
}

impl<T: Default> CallAbsoluteOperation<T> {
    /// Creates a new absolute call operation.
    pub fn new(target_address: usize) -> Self {
        CallAbsoluteOperation {
            scratch_register: Default::default(),
            target_address,
        }
    }
}
