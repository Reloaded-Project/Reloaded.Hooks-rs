extern crate alloc;

use crate::api::errors::hook_builder_error::HookBuilderError;
use alloc::vec::Vec;
use derive_new::new;

/// Represents a target address within memory for allocation nearness.
///
/// This is used for the allocation of wrappers and other native/interop components.
/// It helps guide memory allocations to be closer to a specific target address.
#[derive(new)]
pub struct HookBuilderSettings<TRegister>
where
    TRegister: Clone,
{
    // 👆 boxed to save some code space, not much to gain here perf wise.
    /// The 'source address' used to allocate the stub buffer within proximity of the original code.
    pub source_address: usize,

    /// Retrieves the maximum possible length of the 'stub' buffer.
    ///
    /// # Safety
    ///
    /// This MUST be greater than the data that will be placed in the 'stub' buffer.
    pub max_buf_length: usize,

    /// Retrieves the maximum possible length of the 'swap' space
    /// in the buffer.
    ///
    /// This is the maximum of the original code length and the hook code length.
    pub max_swap_length: usize,

    /// Whether the hook should be activated automatically when it is created.
    /// This should be set to `true`, it is only ever set to 'false' for backwards compatibility
    /// purposes with original C# library.
    ///
    /// When this is set to `false`, the hook will still be 'activated' i.e. the original code will be
    /// overwritten (for thread safety), but the hook will be activated in the disabled state.
    pub auto_activate: bool,

    /// An optional 'scratch register' that can be used to re-encode the original code to a new location.
    /// This is not required for x86, others require it.
    ///
    /// This is only required if platform does not support 'Targeted Memory Allocation', i.e. more
    /// esoteric platforms.
    pub scratch_register: Option<TRegister>,
}

/// Mixin trait for HookBuilder accompanying [`HookBuilderSettings`] that provides
/// custom hook specific functionality.
pub trait HookBuilderSettingsMixin<TRegister> {
    /// Function that retrieves the 'original' code.
    ///
    /// 'Original Code' can differ in definition depending on context.
    ///
    /// For example, for an [`AssemblyHook`][`crate::api::hooks::assembly::assembly_hook::AssemblyHook`], the
    /// 'original' code is the code that was originally at the hook address.
    ///
    /// For a 'branch' hook, it is the code of the branch to be replaced.
    ///
    /// # Parameters
    /// - `address`: Address where the 'original' function should be located.
    /// - `code`: The buffer to receive the 'original' code.
    ///
    /// # Returns
    ///
    /// Returns the original code.
    ///
    /// Can optionally return an error, for example if the hooks has a maximum allowed number of bytes,
    /// you would return [`HookBuilderError::TooManyBytes`].
    fn get_orig_function(
        &mut self,
        address: usize,
        code: &mut Vec<u8>,
    ) -> Result<(), HookBuilderError<TRegister>>;

    /// Function that retrieves the 'hook' code.
    ///
    /// 'Hook Code' can differ in definition depending on context.
    ///
    /// For example, for an [`crate::api::hooks::assembly::assembly_hook::AssemblyHook`], the 'hook'
    /// code is the user's custom code.
    ///
    /// For other hooks, it may either be a branch to the user's custom code, or
    /// a Wrapper/ReverseWrapper.
    ///
    /// # Parameters
    /// - `address`: Address of the hook function.
    /// - `code`: The buffer to receive the hook code.
    ///
    /// # Returns
    ///
    /// Returns the original code.
    ///
    /// Can optionally return an error, for example if the hooks has a maximum allowed number of bytes,
    /// you would return [`HookBuilderError::TooManyBytes`].
    fn get_hook_function(
        &mut self,
        address: usize,
        code: &mut Vec<u8>,
    ) -> Result<(), HookBuilderError<TRegister>>;
}
