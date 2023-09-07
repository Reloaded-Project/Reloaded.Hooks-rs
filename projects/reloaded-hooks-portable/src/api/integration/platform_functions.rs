extern crate alloc;

use core::cell::UnsafeCell;

use crate::api::buffers::{
    buffer_abstractions::BufferFactory, default_buffer_factory::DefaultBufferFactory,
};

use alloc::boxed::Box;

pub struct PlatformFunctions {
    /// The factory for creating read/writable buffers used by the library.
    pub buffer_factory: UnsafeCell<Box<dyn BufferFactory>>,
}

impl Default for PlatformFunctions {
    fn default() -> Self {
        PlatformFunctions {
            buffer_factory: UnsafeCell::new(Box::new(DefaultBufferFactory::new())),
        }
    }
}
