/// Represents an IP-relative call operation, where the address of the target function is located
/// at an offset relative to the current Instruction Pointer (Program Counter).
///
/// # Example
///
/// ```
/// use reloaded_hooks_portable::api::jit::call_rip_relative_operation::CallIpRelativeOperation;
/// let call_op = CallIpRelativeOperation::<i32>::new(0x41FFFC);
///
/// // 0x41FFFC is the location where the address of the target function is located.
///
/// // In x64, this would compile into CALL qword [rip - 4], if assembled at 0x420000.
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallIpRelativeOperation<T> {
    /// Location in memory where the address of the target function is located.
    pub target_address: usize,

    /// Scratch register.
    pub scratch: T,
}

impl<T: Default> CallIpRelativeOperation<T> {
    /// Creates a new IP-relative call operation.
    pub fn new(target_address: usize) -> Self {
        CallIpRelativeOperation {
            target_address,
            scratch: T::default(),
        }
    }
}
