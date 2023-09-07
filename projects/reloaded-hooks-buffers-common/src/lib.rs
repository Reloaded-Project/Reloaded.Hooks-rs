//! # Some Cool Reloaded Library
//! Here's the crate documentation.

use dynamic_link::pointer_functions::*;
use reloaded_memory_buffers::c::buffers_fnptr::BuffersFunctions;

/// Stuff for pointer based dynamic linked (Reloaded3) implementation.
pub mod dynamic_link {
    pub mod pointer_functions;
}

pub mod hooks_buffers_error;
pub mod pointer_linked_buffer;
pub mod pointer_linked_buffer_factory;
pub mod static_linked_buffer;
pub mod static_linked_buffer_factory;

/// Initializes
pub fn init_pointer_functions(functions: &BuffersFunctions) {
    unsafe {
        BUFFERS_GET_BUFFER_ALIGNED = functions.buffers_get_buffer_aligned;
        LOCATORITEM_APPEND_BYTES = functions.locatoritem_append_bytes;
        FREE_GET_BUFFER_RESULT = functions.free_get_buffer_result;
    }
}
