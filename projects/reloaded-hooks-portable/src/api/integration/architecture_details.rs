/// Allows you to get/set details specific to CPU architecture in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchitectureDetails {
    /// Required alignment of code for the current architecture.
    pub code_alignment: u32,

    /// Maximum distance of relative jump assembly instruction.
    /// This affects wrapper generation, and parameters passed into JIT.
    pub max_relative_jump_distance: usize,
}

impl Default for ArchitectureDetails {
    fn default() -> Self {
        ArchitectureDetails {
            /// For non-x86 platforms typically the requested alignment is 4 (or similar).
            /// This information tends to be hard to come by.
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            code_alignment: 4,

            /// Our default code alignment is 16 bytes, as x86 chips fetch instructions in 16 byte chunks.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            code_alignment: 16,

            #[cfg(target_arch = "aarch64")]
            max_relative_jump_distance: u32::MAX as usize,

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            max_relative_jump_distance: i32::MAX as usize,
        }
    }
}
