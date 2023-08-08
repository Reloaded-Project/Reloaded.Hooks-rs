/// Represents an absolute jump operation using a specific register and target address.
///
/// A `JumpAbsolute` specifies which register to use as a base and the target address to jump to.
///
/// # Example
///
/// ```
/// use reloaded_hooks_portable::api::jit::jump_absolute_operation::JumpAbsoluteOperation;
///
/// let jump_op = JumpAbsoluteOperation {
///     scratch_register: "rax",
///     target_address: 0xDEADBEEF,
/// };
///
/// // This might represent an assembly instruction sequence like:
/// //  MOV RAX, 0xDEADBEEF
/// //  JMP RAX
///
/// // Or if scratch_register is not specified, it will represent
/// // jmp qword [0x123456] // at address 0x123456 is 0xDEADBEEF
/// ```
///
/// In a real-world scenario, you'd likely use enums instead of strings for the scratch register,
/// the code above uses a string for demonstration purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpAbsoluteOperation<T> {
    /// The scratch register to use for the jump operation.
    pub scratch_register: T,

    /// The target address to jump to.
    pub target_address: usize,
}

impl<T: Default> JumpAbsoluteOperation<T> {
    /// Creates a new relative jump operation.
    pub fn new(target_address: usize) -> Self {
        JumpAbsoluteOperation {
            scratch_register: Default::default(),
            target_address,
        }
    }
}
