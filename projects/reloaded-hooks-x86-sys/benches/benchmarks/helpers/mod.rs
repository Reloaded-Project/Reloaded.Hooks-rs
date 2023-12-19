use reloaded_memory_buffers::{
    buffers::Buffers,
    structs::{errors::BufferSearchError, params::BufferSearchSettings},
};

#[allow(dead_code)]
pub(crate) type Add = extern "win64" fn(i64, i64) -> i64;

// add_msft_x64.asm
pub const CALCULATOR_ADD_MSFT_X64: [u8; 14] = [
    0x48, 0x89, 0xC8, 0x48, 0x01, 0xD0, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0xC3,
];

pub(crate) fn alloc_function(bytes: &[u8]) -> Result<usize, BufferSearchError> {
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
