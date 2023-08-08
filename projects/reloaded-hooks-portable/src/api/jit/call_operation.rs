/// Represents a call operation using a specific register and target address.
///
/// A `CallOperation` specifies whether to use a relative jump or not,
/// which register to use as a scratch, and the target address to jump to.
///
/// # Example
///
/// In a JIT compilation scenario, you might use this to represent a dynamic
/// call to a target address, leveraging a scratch register.
///
/// ```
/// use reloaded_hooks_portable::api::jit::call_operation::CallOperation;
///
/// let call_op = CallOperation {
///     relative: true,
///     scratch_register: "rax",
///     target_address: 0xDEADBEEF,
/// };
///
/// // This might represent an assembly instruction like: CALL [RAX + 0xDEADBEEF]
/// // if you interpret the address in `RAX` as base for a relative call.
/// println!("CALL {}{}", call_op.scratch_register,
///          if call_op.relative { format!(" + {:X}", call_op.target_address) } else { format!(" {:X}", call_op.target_address) });
/// ```
///
/// Again, in a real-world scenario, you'd likely use enums instead of strings for the scratch register,
/// the code above uses a string for demonstration purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallOperation<T> {
    /// Whether to use a relative jump or not.
    pub relative: bool,

    /// The scratch register to use for the call operation.
    pub scratch_register: T,

    /// The target address to jump to.
    pub target_address: usize,
}

impl<T: Default> CallOperation<T> {
    /// Creates a new relative call operation.
    pub fn make_relative(target_address: usize) -> Self {
        CallOperation {
            relative: false,
            scratch_register: Default::default(),
            target_address,
        }
    }
}

impl<T> CallOperation<T> {
    /// Creates a new absolute call operation.
    pub fn make_absolute(scratch_register: T, target_address: usize) -> Self {
        CallOperation {
            relative: false,
            scratch_register,
            target_address,
        }
    }
}
