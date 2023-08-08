extern crate alloc;

use crate::api::buffers::{
    buffer_abstractions::BufferFactory, default_buffer_factory::DefaultBufferFactory,
};

use alloc::boxed::Box;
use spin::RwLock;

pub struct PlatformFunctions {
    /// The factory for creating read/writable buffers used by the library.
    pub buffer_factory: RwLock<Box<dyn BufferFactory>>,
}

impl Default for PlatformFunctions {
    fn default() -> Self {
        PlatformFunctions {
            buffer_factory: RwLock::new(Box::new(DefaultBufferFactory::new())),
        }
    }
}
