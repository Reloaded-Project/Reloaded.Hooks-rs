/// Represents an absolute jump operation, which reads a value at the given target address
/// into a register, and then jumps to it.
///
/// # Example
///
/// ```
/// use reloaded_hooks_portable::api::jit::jump_absolute_indirect_operation::JumpAbsoluteIndirectOperation;
///
/// let jump_op = JumpAbsoluteIndirectOperation {
///     scratch_register: Some("x9"),
///     pointer_address: 0x123456,
/// };
///
/// // X86:
/// // jmp qword [0x123456] // at address 0x123456 is 0xDEADBEEF
///
/// // ARM64:
/// // MOVZ x9, #0x123, LSL #16
/// // LDR  x9, [x9, #0x456]
/// // BR   x9
/// ```
///
/// In a real-world scenario, you'd likely use enums instead of strings for the scratch register,
/// the code above uses a string for demonstration purposes.
///
/// # Remarks
///
/// JITs are free to encode this as a relative jump if it's possible.
/// This is for cases when architectures have multiple different series of jump instructions they can use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpAbsoluteIndirectOperation<T> {
    /// The scratch register to use for the jump operation.
    pub scratch_register: Option<T>,

    /// The target address to jump to.
    pub pointer_address: usize,
}

impl<T: Default> JumpAbsoluteIndirectOperation<T> {
    /// Creates a new relative jump operation.
    pub fn new(pointer_address: usize) -> Self {
        JumpAbsoluteIndirectOperation {
            scratch_register: None,
            pointer_address,
        }
    }
}
