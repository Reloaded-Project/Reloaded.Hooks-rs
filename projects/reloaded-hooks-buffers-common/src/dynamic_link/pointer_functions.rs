use reloaded_memory_buffers::{
    c::{
        buffers_c_buffers::{buffers_get_buffer_aligned, free_get_buffer_result, GetBufferResult},
        buffers_c_locatoritem::locatoritem_append_bytes,
    },
    structs::{internal::LocatorItem, params::BufferSearchSettings},
};

/// Gets a buffer with user-specified requirements and provided alignment.
///
/// # Arguments
///
/// * `settings` - Settings used for allocating the memory.
/// * `alignment` - The alignment of the buffer. The maximum is 4096, but it's recommended to keep it at or below 64.
///
/// # Returns
///
/// An item that allows you to write to the buffer.
/// Make sure to drop the buffer using the `drop` function after you're done with it.
///
/// # Remarks
///
/// Allocating inside another process is only supported on Windows.
///
/// This function is "dumb"; it will look for a buffer with a size of `settings.size + alignment` and
/// then align it. There's currently no logic that takes alignment into account when searching for a buffer.
/// Contributions are welcome!
///
/// This function might not find some buffers that could accommodate the alignment requirement.
/// It is recommended to use this function when the alignment requirement is less than 64 bytes.
///
/// # Errors
///
/// Returns an error if the memory cannot be allocated within the needed constraints or
/// if there's no existing buffer that satisfies the constraints.
pub(crate) static mut BUFFERS_GET_BUFFER_ALIGNED: extern "C" fn(
    settings: &BufferSearchSettings,
    alignment: u32,
) -> GetBufferResult = buffers_get_buffer_aligned;

/// Appends the data to this buffer.
///
/// # Arguments
///
/// * `item` - The buffer item to which the data will be appended.
/// * `data` - The data that will be appended.
/// * `data_len` - The length of the data to be appended.
///
/// # Returns
///
/// The address where the data has been written.
///
/// # Remarks
///
/// The caller is responsible for ensuring that there is sufficient space in the buffer to hold the data.
/// When returning buffers from the library, the library will ensure there's at least the requested amount of space;
/// so if the total size of your data falls under that space, you should be fine.
///
/// # Safety
///
/// This function is marked as unsafe because the caller must ensure that the buffer is large enough to hold the data.
/// No error will be thrown if the buffer size is insufficient.
pub(crate) static mut LOCATORITEM_APPEND_BYTES: unsafe extern "C" fn(
    item: *mut LocatorItem,
    data: *const u8,
    data_len: usize,
) -> usize = locatoritem_append_bytes;

/// Frees a get buffer result returned from the 'buffers' operation.
///
/// # Arguments
///
/// * `result` - The GetBufferResult object to be freed.
///
/// # Safety
///
/// This function is marked as unsafe as it involves deallocating memory.
pub(crate) static mut FREE_GET_BUFFER_RESULT: unsafe extern "C" fn(GetBufferResult) =
    free_get_buffer_result;
