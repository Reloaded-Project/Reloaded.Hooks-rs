use super::register::Register;

/// This struct defines the calling convention of a function.
pub struct FunctionAttribute {
    /// Registers in left to right parameter order passed to the custom function.
    source_registers: Vec<Register>,

    /// The register that the function returns its value in.
    /// This is typically rax.
    return_register: Register,

    /// Used for allocating an extra amount of uninitialized (not zero-written) stack space
    /// before calling the function. A 32-byte pre-alloc is required by Microsoft x64 calling
    /// convention.
    reserved_stack_Space: u32,

    /// Specifies all the registers whose values are expected to be preserved by the function.
    callee_saved_registers: Vec<Register>,
}
