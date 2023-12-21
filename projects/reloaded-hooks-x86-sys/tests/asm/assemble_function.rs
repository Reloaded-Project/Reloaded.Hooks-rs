use reloaded_memory_buffers::{
    buffers::Buffers,
    structs::{errors::BufferSearchError, params::BufferSearchSettings},
};

pub fn alloc_function(bytes: &[u8]) -> Result<usize, BufferSearchError> {
    let settings = BufferSearchSettings {
        min_address: 0,
        max_address: i32::MAX as usize,
        size: 4096,
    };

    // Automatically dropped.
    let item = Buffers::get_buffer(&settings)?;

    // Append some data.
    unsafe { Ok(item.append_bytes(bytes)) }
}
