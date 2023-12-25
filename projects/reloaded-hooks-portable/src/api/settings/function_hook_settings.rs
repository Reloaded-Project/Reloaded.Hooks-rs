use super::basic_hook_settings::BasicHookSettings;
use crate::api::{
    calling_convention_info::CallingConventionInfo, function_info::FunctionInfo,
    traits::register_info::RegisterInfo,
};

/// Common hook settings for hooks
///
/// This is used for the allocation of wrappers and other native/interop components.
/// It helps guide memory allocations to be closer to a specific target address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionHookSettings<'a, TRegister, TFunctionInfo, TFunctionAttribute>
where
    TRegister: Clone + Copy + RegisterInfo + PartialEq + Eq + 'static,
    TFunctionInfo: FunctionInfo,
    TFunctionAttribute: CallingConventionInfo<TRegister>,
{
    /// Basic settings for the hook.
    pub core_settings: BasicHookSettings<TRegister>,

    /// Whether the hook should be activated automatically when it is created.
    ///
    /// This should be set to `true`, it is only ever set to 'false' for backwards compatibility
    /// purposes with original C# library under some circumstances.
    ///
    /// When this is set to `false`, the hook will still be 'activated' i.e. the original code will be
    /// overwritten (for thread safety), but the hook will be activated in the disabled state.
    pub auto_activate: bool,

    /// Information about the function being hooked,
    /// such as its parameters and return type.
    pub function_info: TFunctionInfo,

    /// Calling convention of the source item (original function).
    pub conv_source: &'a TFunctionAttribute,

    /// Calling convention of the target method (hook function).
    pub conv_target: &'a TFunctionAttribute,

    /// If this parameter is specified, the wrapper will inject an additional parameter
    /// with the specified value into the target (called) function.
    ///
    /// # Remarks
    ///
    /// This is useful for example when the target function is your own method when hooking
    /// and you want to inject a 'this' pointer.
    pub injected_parameter: Option<usize>,
}

impl<'a, TRegister, TFunctionInfo, TFunctionAttribute>
    FunctionHookSettings<'a, TRegister, TFunctionInfo, TFunctionAttribute>
where
    TRegister: Clone + Copy + RegisterInfo + PartialEq + Eq + 'static,
    TFunctionInfo: FunctionInfo,
    TFunctionAttribute: CallingConventionInfo<TRegister>,
{
    /// Determines if a calling convention wrapper is needed for the hook.
    ///
    /// # Returns
    ///
    /// `true` if the calling conventions of the source and target are different,
    /// indicating that a wrapper is required. Otherwise, `false`.
    pub fn needs_wrapper(&self) -> bool {
        self.injected_parameter.is_some() || self.conv_source != self.conv_target
    }
}
