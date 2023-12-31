use super::traits::register_info::RegisterInfo;

use alloc::vec::Vec;
extern crate alloc;

/// This trait defines the calling convention of a function.
///
/// # Generic Parameters
/// - `TRegister`: The type of register used by the target architecture. (Enum)
pub trait CallingConventionInfo<TRegister: Copy + RegisterInfo + PartialEq + 'static>:
    PartialEq
{
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
    /// is reserved AFTER the alignment is made. Therefore, this value must be a multiplier of the
    /// stack alignment.
    fn reserved_stack_space(&self) -> u32;

    /// Specifies all the registers whose values are expected to be preserved by the function.
    ///
    /// # Remarks
    ///
    /// Provide the full sized registers only. For example, if you are on x64, supply `rax`, `rbx`, `rcx`, etc.
    /// Do not supply `eax`, `ax`, `al`, etc,  or match register sizes.
    fn callee_saved_registers(&self) -> &[TRegister];

    /// Specifies all the callee saved registers which will always be preserved by the function; and
    /// excluded from callee saved register elimination.
    ///
    /// # Remarks
    ///
    /// This is usually only used for architectures which have registers which are both caller
    /// and callee saved, such as the link register in RISC architectures.
    ///
    /// Please provide the full sized registers only. For example, if you are on x64, supply `rax`, `rbx`, `rcx`, etc.
    /// Do not supply `eax`, `ax`, `al`, etc,  or match register sizes.
    fn always_saved_registers(&self) -> &[TRegister];

    /// Specifies how the stack is cleaned up after a function call.
    fn stack_cleanup_behaviour(&self) -> StackCleanup;

    /// Specifies the order in which parameters are passed to the stack;
    /// either left-to-right or right-to-left.
    ///
    /// # Remarks
    ///
    /// This field is currently unused.
    fn stack_parameter_order(&self) -> StackParameterOrder;

    /// Required alignment of the stack pointer before the function is called.
    /// This may vary depending on architecture. Tends to be 16 bytes for x64,
    /// 0 bytes for x86, etc.
    fn required_stack_alignment(&self) -> u32;

    /// This is automatically determined based on [`callee_saved_registers`](#method.callee_saved_registers)
    /// and [`always_saved_registers`](#method.always_saved_registers). Returns all registers not listed
    /// there.
    fn caller_saved_registers(&self) -> Vec<TRegister> {
        let callee_saved = self.callee_saved_registers();
        let always_saved = self.always_saved_registers();
        let all_registers = TRegister::all_registers();

        let vec: Vec<TRegister> = all_registers
            .iter()
            .filter(|reg| !callee_saved.contains(reg) && !always_saved.contains(reg))
            .copied()
            .collect();

        vec
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackParameterOrder {
    /// Parameters are pushed onto the stack starting with the rightmost (last) parameter and
    /// proceeding to the left. This is common in many C and C++ calling conventions on platforms
    /// like x86.
    RightToLeft,

    /// This is currently not supported in this library, and will throw an error.
    LeftToRight,
}

/// Base struct representing the calling convention of a function, detailing how
/// parameters are passed, which registers are used, and how the stack is managed.
///
/// This struct is useful for defining different calling conventions in a way that
/// can be easily referenced and utilized in various contexts, such as JIT compilation
/// or function hooking.
///
/// # Fields
///
/// - `int_parameters`: A slice of registers used for passing integer parameters.
/// - `float_parameters`: A slice of registers used for passing floating-point parameters.
/// - `vector_parameters`: A slice of registers used for passing vector parameters.
/// - `return_register`: The register used for the function return value.
/// - `reserved_stack_space`: The amount of stack space reserved for the function.
/// - `callee_saved_registers`: Registers that the callee is responsible for saving and restoring.
/// - `always_saved_registers`: Registers that are always saved across function calls.
/// - `stack_cleanup`: Specifies who cleans up the stack after the function call.
/// - `stack_parameter_order`: The order in which parameters are pushed onto the stack.
/// - `required_stack_alignment`: The required alignment of the stack pointer before the function call.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericCallingConvention<'a, TRegister: Copy> {
    pub int_parameters: &'a [TRegister],
    pub float_parameters: &'a [TRegister],
    pub vector_parameters: &'a [TRegister],
    pub return_register: TRegister,
    pub reserved_stack_space: u32,
    pub callee_saved_registers: &'a [TRegister],
    pub always_saved_registers: &'a [TRegister],
    pub stack_cleanup: StackCleanup,
    pub stack_parameter_order: StackParameterOrder,
    pub required_stack_alignment: u32,
}

impl<'a, TRegister: Copy + RegisterInfo + PartialEq + 'static> CallingConventionInfo<TRegister>
    for GenericCallingConvention<'a, TRegister>
{
    fn register_int_parameters(&self) -> &[TRegister] {
        self.int_parameters
    }

    fn register_float_parameters(&self) -> &[TRegister] {
        self.float_parameters
    }

    fn register_vector_parameters(&self) -> &[TRegister] {
        self.vector_parameters
    }

    fn return_register(&self) -> TRegister {
        self.return_register
    }

    fn reserved_stack_space(&self) -> u32 {
        self.reserved_stack_space
    }

    fn callee_saved_registers(&self) -> &[TRegister] {
        self.callee_saved_registers
    }

    fn always_saved_registers(&self) -> &[TRegister] {
        self.always_saved_registers
    }

    fn stack_cleanup_behaviour(&self) -> StackCleanup {
        self.stack_cleanup
    }

    fn stack_parameter_order(&self) -> StackParameterOrder {
        self.stack_parameter_order
    }

    fn required_stack_alignment(&self) -> u32 {
        self.required_stack_alignment
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        api::calling_convention_info::CallingConventionInfo,
        helpers::test_helpers::{
            MockRegister::{self, *},
            CDECL_LIKE_FUNCTION_ATTRIBUTE,
        },
    };

    #[test]
    fn cdecl_like_saved_registers() {
        // Removed both always saved (LR, SP), and callee saved (R3, R4, F3, F4, V3, V4).
        let mut caller_saved = CDECL_LIKE_FUNCTION_ATTRIBUTE.caller_saved_registers();
        let mut expected: Vec<MockRegister> = vec![R0, R1, R2, F0, F1, F2, V0, V1, V2, SP, LR];
        caller_saved.sort();
        expected.sort();
        assert_eq!(caller_saved, expected);
    }
}
