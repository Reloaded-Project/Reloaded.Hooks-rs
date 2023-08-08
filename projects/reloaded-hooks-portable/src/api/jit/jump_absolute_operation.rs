/// Represents an absolute jump operation using a specific register and target address.
///
/// A `JumpAbsolute` specifies which register to use as a base and the target address to jump to.
///
/// # Example
///
/// ```
/// use reloaded_hooks_portable::api::jit::jump_operation::JumpAbsolute;
///
/// let jump_op = JumpAbsolute {
///     scratch_register: "rax",
///     target_address: 0xDEADBEEF,
/// };
///
/// // This might represent an assembly instruction like: JMP [RAX + 0xDEADBEEF]
/// println!("JMP {} +{:X}", jump_op.scratch_register, jump_op.target_address);
/// ```
///
/// Again, in a real-world scenario, you'd likely use enums instead of strings for the scratch register,
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
