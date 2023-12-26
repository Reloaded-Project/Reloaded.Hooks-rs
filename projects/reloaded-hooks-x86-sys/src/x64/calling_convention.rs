extern crate alloc;

use super::Register;
use super::Register::*;
use derive_more::Deref;
use derive_more::DerefMut;
use reloaded_hooks_portable::api::calling_convention_info::{
    GenericCallingConvention, StackCleanup, StackParameterOrder,
};

/// A variant of `GenericCallingConvention` for x64.
///
/// This struct is specialized for x64 and includes commonly used calling conventions such
/// as Microsoft x64 and System V AMD64.
///
/// # Examples
///
/// ```rust
/// use reloaded_hooks_x86_sys::x64::calling_convention::CallingConvention;
/// let microsoft_x64_convention = CallingConvention::microsoft_x64();
/// ```
#[derive(Debug, Clone, PartialEq, DerefMut, Deref)]
pub struct CallingConvention<'a> {
    convention: GenericCallingConvention<'a, Register>,
}

static MICROSOFT_X64: CallingConvention = CallingConvention {
    convention: GenericCallingConvention::<Register> {
        int_parameters: &[rcx, rdx, r8, r9],
        float_parameters: &[xmm0, xmm1, xmm2, xmm3],
        vector_parameters: &[],
        return_register: rax,
        reserved_stack_space: 32, // 'shadow space'
        callee_saved_registers: &[
            rbp, rbx, rdi, rsi, r12, r13, r14, r15, xmm6, xmm7, xmm8, xmm9, xmm10, xmm11, xmm12,
            xmm13, xmm14, xmm15,
        ],
        always_saved_registers: &[],
        stack_cleanup: StackCleanup::Callee,
        stack_parameter_order: StackParameterOrder::RightToLeft,
        required_stack_alignment: 16,
    },
};

// https://gitlab.com/x86-psABIs/x86-64-ABI/-/jobs/5301578287/artifacts/raw/x86-64-ABI/abi.pdf
// Parameter passing section
static SYSTEM_V_AMD64: CallingConvention = CallingConvention {
    convention: GenericCallingConvention::<Register> {
        int_parameters: &[rdi, rsi, rdx, rcx, r8, r9],
        float_parameters: &[xmm0, xmm1, xmm2, xmm3, xmm4, xmm5, xmm6, xmm7],
        vector_parameters: &[],
        return_register: rax,
        reserved_stack_space: 0, // 'red zone' is on the other side of the stack pointer, as opposed to 'shadow space'.
        callee_saved_registers: &[rbp, rbx, r12, r13, r14, r15],
        always_saved_registers: &[],
        stack_cleanup: StackCleanup::Caller,
        stack_parameter_order: StackParameterOrder::RightToLeft,
        required_stack_alignment: 16,
    },
};

impl<'a> CallingConvention<'a> {
    /// Returns an instance of the CallingConvention struct configured for the
    /// Microsoft x64 calling convention, commonly used in Windows.
    pub fn microsoft_x64() -> &'a Self {
        &MICROSOFT_X64
    }

    /// Returns an instance of the CallingConvention struct configured for the
    /// System V AMD64 calling convention, commonly used in Unix-like systems.
    pub fn system_v() -> &'a Self {
        &SYSTEM_V_AMD64
    }

    /// Returns a [`CallingConvention`] based on the provided [`PresetCallingConvention`].
    pub fn from_preset(convention_type: PresetCallingConvention) -> &'a Self {
        match convention_type {
            PresetCallingConvention::MicrosoftX64 => Self::microsoft_x64(),
            PresetCallingConvention::SystemV => Self::system_v(),
        }
    }

    /// Returns the default calling convention for the current platform.
    /// This method uses conditional compilation to determine the appropriate
    /// convention based on the target architecture and operating system.
    pub fn default_for_current_platform() -> &'a Self {
        #[cfg(windows)]
        {
            Self::microsoft_x64()
        }

        #[cfg(not(windows))]
        {
            Self::system_v()
        }
    }
}

/// Enum representing various x64 calling conventions with detailed information.
pub enum PresetCallingConvention {
    /// Microsoft x64 calling convention (used in Windows).
    ///
    /// - Integer parameters: RCX, RDX, R8, R9 (in order, left to right)
    /// - Float parameters:   XMM0 to XMM3 (in order, left to right)
    /// - Additional parameters: Pushed onto stack right to left
    /// - Return register:    RAX (integer), XMM0 (float)
    /// - Cleanup:            Callee
    MicrosoftX64,

    /// System V AMD64 ABI calling convention (used in Unix-like systems).
    ///
    /// - Integer parameters: RDI, RSI, RDX, RCX, R8, R9 (in order, left to right)
    /// - Float parameters:   XMM0 to XMM7 (in order, left to right)
    /// - Additional parameters: Pushed onto stack right to left
    /// - Return register:    RAX (integer), XMM0 (float)
    /// - Cleanup:            Caller
    SystemV,
}
