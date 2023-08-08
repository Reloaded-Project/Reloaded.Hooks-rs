/// Represents an IP-relative jump operation, where the address of the target location is located
/// at an offset relative to the current Instruction Pointer (Program Counter).
///
/// # Example
///
/// ```
/// use reloaded_hooks_portable::api::jit::jump_rip_relative_operation::JumpIpRelativeOperation;
/// let jump_op = JumpIpRelativeOperation { target_address: 0x41FFFC };
///
/// // 0x41FFFC is the location where the address of the target location is stored.
///
/// // In x64, this would compile into JMP qword [rip - 4], if assembled at 0x420000.
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpIpRelativeOperation {
    /// Location in memory where the address of the target location is located.
    pub target_address: usize,
}

impl JumpIpRelativeOperation {
    /// Creates a new IP-relative jump operation.
    pub fn new(target_address: usize) -> Self {
        JumpIpRelativeOperation { target_address }
    }
}
