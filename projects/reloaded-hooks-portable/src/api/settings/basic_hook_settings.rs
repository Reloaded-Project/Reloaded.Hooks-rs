/// 'Basic' settings shared between all hooking APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasicHookSettings<TRegister: Copy>
where
    TRegister: Clone,
{
    /// Address of the code to be hooked.
    pub hook_address: usize,

    /// The new location the code should point to.
    pub new_target: usize,

    /// An optional 'scratch register' that can be used to re-encode the original code to a new location.
    /// This is not required for x86, others require it.
    ///
    /// This is only required if platform does not support 'Targeted Memory Allocation', i.e. more
    /// esoteric platforms.
    pub scratch_register: Option<TRegister>,
}

impl<TRegister: Copy> BasicHookSettings<TRegister>
where
    TRegister: Clone,
{
    /// Creates a new `HookBasicSettings` instance with the essential parameters.
    ///
    /// # Parameters
    /// - `hook_address`: Address of the branch to be hooked.
    /// - `new_target`: The new location the branch should point to.
    ///
    /// # Returns
    /// Returns a new instance of `HookBasicSettings` with default values for optional fields.
    pub fn new_minimal(hook_address: usize, new_target: usize) -> Self {
        Self {
            hook_address,
            new_target,
            scratch_register: None,
        }
    }

    /// Creates a new `HookBasicSettings` instance with all parameters, including a scratch register.
    ///
    /// # Parameters
    /// - `hook_address`: Address of the branch to be hooked.
    /// - `new_target`: The new location the branch should point to.
    /// - `scratch_register`: An optional register for re-encoding.
    ///
    /// # Returns
    /// Returns a new instance of `HookBasicSettings` with all provided values.
    pub fn new_with_scratch_register(
        hook_address: usize,
        new_target: usize,
        scratch_register: Option<TRegister>,
    ) -> Self {
        Self {
            hook_address,
            new_target,
            scratch_register,
        }
    }
}
