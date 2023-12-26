use crate::all_registers::AllRegisters;
use crate::all_registers::AllRegisters::*;
use derive_more::Deref;
use derive_more::DerefMut;
use reloaded_hooks_portable::api::calling_convention_info::GenericCallingConvention;
use reloaded_hooks_portable::api::calling_convention_info::StackCleanup;
use reloaded_hooks_portable::api::calling_convention_info::StackParameterOrder;

/// A variant of `GenericCallingConvention` for ARM64.
///
/// This struct is specialized for ARM64 and includes commonly used calling conventions such
/// as AAPCS64.
///
/// # Examples
///
/// ```rust
/// use reloaded_hooks_arm64_sys::arm64::calling_conventions::CallingConvention;
/// let aapcs64_convention = CallingConvention::aapcs64();
/// ```
#[derive(Debug, Clone, PartialEq, DerefMut, Deref)]
pub struct CallingConvention<'a> {
    convention: GenericCallingConvention<'a, AllRegisters>,
}

static AAPCS64: CallingConvention = CallingConvention {
    convention: GenericCallingConvention::<AllRegisters> {
        int_parameters: &[x0, x1, x2, x3, x4, x5, x6, x7],
        float_parameters: &[v0, v1, v2, v3, v4, v5, v6, v7],
        vector_parameters: &[],
        return_register: x0, // Assuming x0 is for integers and v0 for floats
        reserved_stack_space: 0,
        callee_saved_registers: &[x19, x20, x21, x22, x23, x24, x25, x26, x27, x28, x29],
        always_saved_registers: &[LR],
        stack_cleanup: StackCleanup::Caller,
        stack_parameter_order: StackParameterOrder::RightToLeft,
        required_stack_alignment: 16, // mandated by hardware
    },
};

// https://learn.microsoft.com/en-us/cpp/build/arm64-windows-abi-conventions?view=msvc-170
static MICROSOFT_ARM64: CallingConvention = CallingConvention {
    convention: GenericCallingConvention::<AllRegisters> {
        int_parameters: &[x0, x1, x2, x3, x4, x5, x6, x7, x8], // not a typo, has x8
        float_parameters: &[v0, v1, v2, v3, v4, v5, v6, v7],
        vector_parameters: &[],
        return_register: x0,
        reserved_stack_space: 16, // Documented as 'Red zone'
        callee_saved_registers: &[x19, x20, x21, x22, x23, x24, x25, x26, x27, x28, x29],
        always_saved_registers: &[LR],
        stack_cleanup: StackCleanup::Caller,
        stack_parameter_order: StackParameterOrder::RightToLeft,
        required_stack_alignment: 16, // mandated by hardware
    },
};

impl<'a> CallingConvention<'a> {
    /// ARM64 AAPCS64 calling convention.
    /// - Integer parameters: X0 to X7 for the first eight integer or pointer arguments.
    /// - Float parameters:   V0 to V7 for the first eight floating-point arguments.
    /// - Additional parameters: Passed on the stack.
    /// - Return register:    X0 (integer), V0 (float)
    /// - Cleanup:            Caller
    pub fn aapcs64() -> &'a Self {
        &AAPCS64
    }

    /// Microsoft ARM64 calling convention.
    /// - Integer parameters: X0 to X8 for the first eight integer or pointer arguments.
    /// - Float parameters:   V0 to V7 for the first eight floating-point arguments.
    /// - Additional parameters: Passed on the stack.
    /// - Return register:    X0 (integer), V0 (float)
    /// - Cleanup:            Caller
    ///
    /// Has 16 bytes of 'red zone'
    pub fn microsoft() -> &'a Self {
        &MICROSOFT_ARM64
    }

    // Add a method to select ARM64 calling convention based on PresetCallingConvention
    pub fn from_preset(convention_type: PresetCallingConvention) -> &'a Self {
        match convention_type {
            PresetCallingConvention::AAPCS64 => Self::aapcs64(),
            PresetCallingConvention::Microsoft => Self::microsoft(),
        }
    }

    // Retrieves the default calling convention for the currently running machine.
    pub fn default_for_current_platform() -> &'a Self {
        #[cfg(windows)]
        {
            Self::microsoft()
        }

        #[cfg(not(windows))]
        {
            Self::aapcs64()
        }
    }
}

/// Enum representing various calling conventions with detailed information.
pub enum PresetCallingConvention {
    /// ARM64 AAPCS64 calling convention.
    /// - Integer parameters: X0 to X7 for the first eight integer or pointer arguments.
    /// - Float parameters:   V0 to V7 for the first eight floating-point arguments.
    /// - Additional parameters: Passed on the stack.
    /// - Return register:    X0 (integer), V0 (float)
    /// - Cleanup:            Caller
    AAPCS64,

    /// Microsoft ARM64 calling convention.
    /// - Integer parameters: X0 to X8 for the first eight integer or pointer arguments.
    /// - Float parameters:   V0 to V7 for the first eight floating-point arguments.
    /// - Additional parameters: Passed on the stack.
    /// - Return register:    X0 (integer), V0 (float)
    /// - Cleanup:            Caller
    ///
    /// Has 16 bytes of 'red zone'
    Microsoft,
}
