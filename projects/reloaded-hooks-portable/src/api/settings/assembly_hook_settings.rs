/// Represents a target address within memory for allocation nearness.
///
/// This is used for the allocation of wrappers and other native/interop components.
/// It helps guide memory allocations to be closer to a specific target address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssemblyHookSettings<'a> {
    /// Address of the function or mid function to be hooked.
    pub hook_address: u32,

    /// The assembly code to be emplaced at the hook address.
    pub asm_code: &'a [u8],

    /// The 'original' address of the assembly code contained in the [`AssemblyHookSettings::asm_code`] field.
    ///
    /// If this field is set to a value other than 0, then the code in [`AssemblyHookSettings::asm_code`] will be re-encoded
    /// before being written
    pub asm_code_address: usize,

    /// Maximum amount of bytes that a `jump` placed at the hook address can handle.
    /// Values larger than this will cause an error to be returned from the hook function.
    pub max_permitted_bytes: usize,

    /// Defines the behaviour of the assembly hook. This can be executing your code before original,
    /// after original or not executing original at all.
    pub behaviour: AsmHookBehaviour,

    /// Whether the hook should be activated automatically when it is created.
    /// This should be set to `true`, it is only ever set to 'false' for backwards compatibility
    /// purposes with original C# library.
    pub auto_activate: bool,
}

impl<'a> AssemblyHookSettings<'a> {
    /// Creates a new `AssemblyHookSettings` instance with the basic parameters.
    /// The assembly code will be executed before the original code at the hook address.
    ///
    /// # Parameters
    /// - `hook_address`: Address of the function or mid-function to be hooked.
    /// - `asm_code`: The assembly code to be emplaced at the hook address.
    /// - `max_permitted_bytes`: Maximum amount of bytes that a `jump` placed at the hook address can handle.
    pub fn new_minimal(hook_address: u32, asm_code: &'a [u8], max_permitted_bytes: usize) -> Self {
        AssemblyHookSettings {
            hook_address,
            asm_code,
            asm_code_address: 0, // Default value for optional field
            max_permitted_bytes,
            behaviour: AsmHookBehaviour::ExecuteFirst,
            auto_activate: true,
        }
    }

    /// Creates a new `AssemblyHookSettings` instance with behaviour specification.
    ///
    /// # Parameters
    /// - `hook_address`: Address of the function or mid-function to be hooked.
    /// - `asm_code`: The assembly code to be emplaced at the hook address.
    /// - `max_permitted_bytes`: Maximum amount of bytes that a `jump` placed at the hook address can handle.
    /// - `behaviour`: Defines when the assembly code will be executed in relation to the original code.
    pub fn new_with_behaviour(
        hook_address: u32,
        asm_code: &'a [u8],
        max_permitted_bytes: usize,
        behaviour: AsmHookBehaviour,
    ) -> Self {
        AssemblyHookSettings {
            hook_address,
            asm_code,
            asm_code_address: 0,
            max_permitted_bytes,
            behaviour,
            auto_activate: true,
        }
    }

    /// Creates a new `AssemblyHookSettings` instance with an additional parameter for the original code address.
    ///
    /// # Parameters
    /// - `hook_address`: Address of the function or mid-function to be hooked.
    /// - `asm_code`: The assembly code to be emplaced at the hook address.
    /// - `asm_code_address`: Original address of the assembly code, if different from the hook address.
    /// - `max_permitted_bytes`: Maximum amount of bytes that a `jump` placed at the hook address can handle.
    pub fn new_with_asm_code_address(
        hook_address: u32,
        asm_code: &'a [u8],
        asm_code_address: usize,
        max_permitted_bytes: usize,
    ) -> Self {
        AssemblyHookSettings {
            hook_address,
            asm_code,
            asm_code_address,
            max_permitted_bytes,
            behaviour: AsmHookBehaviour::ExecuteFirst,
            auto_activate: true,
        }
    }

    /// Creates a new `AssemblyHookSettings` instance with all available parameters.
    ///
    /// # Parameters
    /// - `hook_address`: Address of the function or mid-function to be hooked.
    /// - `asm_code`: The assembly code to be emplaced at the hook address.
    /// - `asm_code_address`: Original address of the assembly code, if different from the hook address.
    /// - `max_permitted_bytes`: Maximum amount of bytes that a `jump` placed at the hook address can handle.
    /// - `behaviour`: Defines when the assembly code will be executed in relation to the original code.
    pub fn new(
        hook_address: u32,
        asm_code: &'a [u8],
        asm_code_address: usize,
        max_permitted_bytes: usize,
        behaviour: AsmHookBehaviour,
    ) -> Self {
        AssemblyHookSettings {
            hook_address,
            asm_code,
            asm_code_address,
            max_permitted_bytes,
            behaviour,
            auto_activate: true,
        }
    }
}

/// Defines the behaviour used by the `AssemblyHook`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsmHookBehaviour {
    /// Executes your assembly code before the original.
    ExecuteFirst,

    /// Executes your assembly code after the original.
    ExecuteAfter,

    /// Do not execute original replaced code. (Dangerous!)
    DoNotExecuteOriginal,
}
