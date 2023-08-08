/// Represents an exchange operation which swaps the contents of two registers.
///
/// This can be used to model assembly `xchg` instructions.
///
/// # Fields
///
/// `register1`, `register2`: These are the two registers that are being exchanged. The type
/// of the registers is generic (`T`), so it can be defined to fit the particular
/// architecture or simulation you're working with.
///
/// # Example
///
/// The `XChgOperation` can be used to represent assembly `xchg` instructions.
/// For instance, in x86 assembly, the instruction `xchg eax, ebx` would swap the values
/// of the `eax` and `ebx` registers. This can be modeled using `XChgOperation`
/// as follows:
///
/// ```
/// use reloaded_hooks_portable::api::jit::xchg_operation::XChgOperation;
/// let xchg = XChgOperation
/// {
///     register1: "eax",
///     register2: "ebx",
///     #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
///     scratch: None,
/// };
/// ```
///
/// # Note
///
/// The `XChgOperation` only represents the operation itself; it does not perform
/// the operation or modify any actual register values. To simulate the
/// effect of the operation, you would need to implement additional logic or use
/// a computing architecture simulation framework.
///
/// In the real world, you should use enums instead of strings for register1 and register2,
/// the code above shows strings for clarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XChgOperation<T> {
    /// The first register to exchange.
    pub register1: T,
    /// The second register to exchange.
    pub register2: T,

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    /// A scratch register to use (applies only to architectures without explicit ).
    pub scratch: Option<T>,
}
