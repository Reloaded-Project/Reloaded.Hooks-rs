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

impl<'a> CallingConvention<'a> {
    /// ARM64 AAPCS64 calling convention.
    /// - Integer parameters: X0 to X7 for the first eight integer or pointer arguments.
    /// - Float parameters:   V0 to V7 for the first eight floating-point arguments.
    /// - Additional parameters: Passed on the stack.
    /// - Return register:    X0 (integer), V0 (float)
    /// - Cleanup:            Caller
    pub fn aapcs64() -> Self {
        static INT_PARAMS: [AllRegisters; 8] = [x0, x1, x2, x3, x4, x5, x6, x7];
        static FLOAT_PARAMS: [AllRegisters; 8] = [v0, v1, v2, v3, v4, v5, v6, v7];
        static CALLEE_SAVED: [AllRegisters; 11] =
            [x19, x20, x21, x22, x23, x24, x25, x26, x27, x28, x29];
        static ALWAYS_SAVED: [AllRegisters; 1] = [LR];

        CallingConvention {
            convention: GenericCallingConvention::<AllRegisters> {
                int_parameters: &INT_PARAMS,
                float_parameters: &FLOAT_PARAMS,
                vector_parameters: &[],
                return_register: x0,
                reserved_stack_space: 0,
                callee_saved_registers: &CALLEE_SAVED,
                always_saved_registers: &ALWAYS_SAVED,
                stack_cleanup: StackCleanup::Caller,
                stack_parameter_order: StackParameterOrder::RightToLeft,
                required_stack_alignment: 16,
            },
        }
    }

    // Add a method to select ARM64 calling convention based on PresetCallingConvention
    pub fn from_preset(convention_type: PresetCallingConvention) -> Self {
        match convention_type {
            PresetCallingConvention::AAPCS64 => Self::aapcs64(),
            _ => unimplemented!(),
        }
    }
}

impl Default for CallingConvention<'_> {
    fn default() -> Self {
        Self::aapcs64()
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
}
