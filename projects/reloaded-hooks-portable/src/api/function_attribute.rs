extern crate alloc;

/// This trait defines the calling convention of a function.
///
/// # Generic Parameters
/// - `TRegister`: The type of register used by the target architecture. (Enum)
pub trait FunctionAttribute<TRegister> {
    /// Registers in left to right parameter order passed to the custom function.
    fn register_int_parameters(&self) -> &[TRegister];

    /// Float registers in left to right parameter order passed to the custom function.
    fn register_float_parameters(&self) -> &[TRegister];

    /// Vector registers in left to right parameter order passed to the custom function.
    fn register_vector_parameters(&self) -> &[TRegister];

    /// The register that the function returns its value in.
    /// In x86 this is typically 'eax/rax'.
    ///
    /// # Remarks
    /// This is not necessarily the same as the register that the function returns its value in.
    fn return_register(&self) -> TRegister;

    /// Used for allocating an extra amount of uninitialized (not zero-written) stack space
    /// before calling the function. This is useful for functions that use the stack for temporary storage.
    ///
    /// # Remarks
    /// Some calling conventions and/or ABIs require stack alignment. In those cases, this stack space
    /// is reserved BEFORE the alignment is made. If the value of this variable is less than the
    /// ABI alignment requirement.
    fn reserved_stack_space(&self) -> u32;

    /// Specifies all the registers whose values are expected to be preserved by the function.
    fn callee_saved_registers(&self) -> &[TRegister];

    /// Specifies all the callee saved registers which will always be preserved by the function; and
    /// excluded from callee saved register elimination.
    ///
    /// # Remarks
    ///
    /// This is usually only used for architectures which have registers which are both caller
    /// and callee saved, such as the link register in RISC architectures.
    fn always_saved_registers(&self) -> &[TRegister];

    /// Specifies how the stack is cleaned up after a function call.
    fn stack_cleanup_behaviour(&self) -> StackCleanup;

    /// Specifies the order in which parameters are passed to the stack;
    /// either left-to-right or right-to-left.
    fn stack_parameter_order(&self) -> StackParameterOrder;

    /// Required alignment of the stack pointer before the function is called.
    /// This may vary depending on architecture. Tends to be 16 bytes for x64,
    /// 0 bytes for x86, etc.
    fn required_stack_alignment(&self) -> usize;
}

/// Defines how the calling convention cleans up the stack after a function call.
///
/// Different calling conventions dictate whether the function that has been
/// called (the "callee") or the function that did the calling (the "caller")
/// is responsible for cleaning up the stack.
///
/// This cleanup involves adjusting the stack pointer to release space that
/// was used for arguments passed to the function.
///
/// # Variants
///
/// - `Caller`: Indicates that the function making the call (the caller) is
/// responsible for cleaning up the stack after the function call returns.
/// This is common in conventions like the cdecl used in many C compilers
/// for x86 architecture.
///
/// - `Callee`: Indicates that the function being called (the callee) will clean
/// up its own arguments from the stack before it returns. This is seen in
/// conventions like stdcall used in the Windows API.
pub enum StackCleanup {
    /// Indicates that the function making the call (the caller) is
    /// responsible for cleaning up the stack after the function call returns.
    /// This is common in conventions like the cdecl used in many C compilers
    /// for x86 architecture. (e.g. sub esp, 8)
    Caller,

    /// Indicates that the function being called (the callee) will clean
    /// up its own arguments from the stack before it returns. This is seen in
    /// conventions like stdcall used in the Windows API. (e.g. ret 8)
    Callee,
}

/// Represents the order in which function parameters are pushed onto the stack when making a function call.
///
/// In some calling conventions, parameters are pushed onto the stack from right to left, meaning
/// the last (rightmost) parameter is pushed first. In others, parameters are pushed from left to right.
/// The order can affect how functions are called and how stack cleanup is performed.
///
/// This distinction is especially important when interfacing with different foreign function interfaces
/// or when writing low-level code that manipulates the stack directly.
pub enum StackParameterOrder {
    /// Parameters are pushed onto the stack starting with the rightmost (last) parameter and
    /// proceeding to the left. This is common in many C and C++ calling conventions on platforms
    /// like x86.
    RightToLeft,

    /// This is currently not supported in this library, and will throw an error.
    LeftToRight,
}
