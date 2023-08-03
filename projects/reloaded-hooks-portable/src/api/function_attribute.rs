extern crate alloc;
use alloc::vec::Vec;

/// This struct defines the calling convention of a function.
///
/// # Generic Parameters
/// - `TRegister`: The type of register used by the target architecture. (Enum)
pub struct FunctionAttribute<TRegister> {
    /// Registers in left to right parameter order passed to the custom function.
    source_registers: Vec<TRegister>,

    /// The register that the function returns its value in.
    /// In x86 this is typically 'eax/rax'.
    return_register: TRegister,

    /// Used for allocating an extra amount of uninitialized (not zero-written) stack space
    /// before calling the function.
    reserved_stack_space: u32,

    /// Specifies all the registers whose values are expected to be preserved by the function.
    callee_saved_registers: Vec<TRegister>,
}
