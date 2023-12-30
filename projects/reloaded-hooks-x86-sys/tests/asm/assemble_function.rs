use reloaded_memory_buffers::{
    buffers::Buffers,
    structs::{errors::BufferSearchError, params::BufferSearchSettings},
};

pub fn alloc_function(bytes: &[u8]) -> Result<usize, BufferSearchError> {
    #[cfg(target_pointer_width = "64")]
    let settings = BufferSearchSettings {
        min_address: u32::MAX as usize,
        max_address: u32::MAX as usize * 2,
        size: 4096,
    };

    #[cfg(target_pointer_width = "32")]
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

pub fn alloc_function_with_settings(
    bytes: &[u8],
    settings: BufferSearchSettings,
) -> Result<usize, BufferSearchError> {
    // Automatically dropped.
    let item = Buffers::get_buffer(&settings)?;

    // Append some data.
    unsafe { Ok(item.append_bytes(bytes)) }
}
