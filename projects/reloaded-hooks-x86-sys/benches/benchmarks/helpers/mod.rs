use reloaded_memory_buffers::{
    buffers::Buffers,
    structs::{errors::BufferSearchError, params::BufferSearchSettings},
};

#[allow(dead_code)]
pub(crate) type Add = extern "win64" fn(i64, i64) -> i64;

// add_msft_x64.asm
#[allow(dead_code)]
pub const CALCULATOR_ADD_CDECL_X86: [u8; 11] = [
    0x55, 0x89, 0xE5, 0x8B, 0x45, 0x08, 0x03, 0x45, 0x0C, 0x5D, 0xC3,
];

#[allow(dead_code)]
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
