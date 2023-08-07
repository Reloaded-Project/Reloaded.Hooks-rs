extern crate alloc;
use alloc::boxed::Box;
use spin::RwLock;

use crate::structs::operation::Operation;

use super::buffers::{
    buffer_abstractions::BufferFactory, default_buffer_factory::DefaultBufferFactory,
};

static mut ARCHITECTURE_DETAILS: RwLock<Option<ArchitectureDetails>> = RwLock::new(None);
static mut FUNCTIONS: RwLock<Option<PlatformFunctions>> = RwLock::new(None);

/// Allows you to get/set details specific to CPU architecture in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchitectureDetails {
    /// Required alignment of code for the current architecture.
    pub code_alignment: u32,

    /// Maximum distance of relative jump assembly instruction.
    /// This affects wrapper generation, and parameters passed into JIT.
    pub max_relative_jump_distance: usize,
}

/// Functions used by the library.
pub struct PlatformFunctions {
    /// The factory for creating read/writable buffers used by the library.
    pub buffer_factory: Box<dyn BufferFactory>,
}

impl Default for PlatformFunctions {
    fn default() -> Self {
        PlatformFunctions {
            buffer_factory: Box::new(DefaultBufferFactory::new()),
        }
    }
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

/// Get or initialize the global functions.
pub fn get_platform_mut() -> spin::RwLockWriteGuard<'static, Option<PlatformFunctions>> {
    unsafe {
        let mut write_lock = FUNCTIONS.write();
        if write_lock.is_none() {
            *write_lock = Some(PlatformFunctions::default());
        }

        write_lock
    }
}

/// Get or initialize the global architecture details.
pub fn set_architecture_details(details: &ArchitectureDetails) {
    unsafe {
        let mut write_lock = ARCHITECTURE_DETAILS.write();
        *write_lock = Some(*details);
    }
}

/// Gets a copy of the architecture details.
pub fn get_architecture_details() -> ArchitectureDetails {
    unsafe {
        // Check if details are already set
        {
            let read_lock = ARCHITECTURE_DETAILS.read();
            if let Some(details) = &*read_lock {
                return *details;
            }
        }

        // If details weren't set, initialize and return them
        let mut write_lock = ARCHITECTURE_DETAILS.write();
        if write_lock.is_none() {
            *write_lock = Some(ArchitectureDetails::default());
        }

        *write_lock.as_ref().unwrap_unchecked()
    }
}
