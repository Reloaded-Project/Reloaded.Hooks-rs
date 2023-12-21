use crate::api::buffers::buffer_abstractions::Buffer;
use core::{hint::unreachable_unchecked, ptr::read_unaligned};

pub trait AtomicWriter {
    /// Writes a native integer type to a given address atomically.
    ///
    /// # Parameters
    ///
    /// - `address`: The address to overwrite.
    /// - `value`: The value to write.
    fn atomic_write<TInteger>(address: usize, value: TInteger)
    where
        Self: Sized;
}

pub struct NativeMemoryAtomicWriter {}

impl AtomicWriter for NativeMemoryAtomicWriter {
    fn atomic_write<TInteger>(address: usize, value: TInteger)
    where
        Self: Sized,
    {
        unsafe {
            *(address as *mut TInteger) = value;
        }
    }
}

/// Automatic implementation for buffers.
impl<T> AtomicWriter for T
where
    T: Buffer,
{
    fn atomic_write<TInteger>(address: usize, value: TInteger)
    where
        Self: Sized,
    {
        T::overwrite_atomic(address, value);
    }
}

pub const MAX_ATOMIC_WRITE_BYTES: u8 = 16;

/// Overwrites bytes at the specified address with provided bytes using atomic operations.
/// This will only replace the specified number of bytes, preserving the rest.
///
/// # Safety
///
/// Readable memory at 'address' must be at least 'num_bytes' rounded up to next power of 2 long.
/// I.e. This may not work if at end of virtual address space.
#[inline]
pub unsafe fn atomic_write_masked<TWriter>(address: usize, code: &[u8], num_bytes: usize)
where
    TWriter: AtomicWriter,
{
    unsafe {
        match num_bytes {
            1 => {
                TWriter::atomic_write(address, *code.as_ptr());
            }
            2 => {
                TWriter::atomic_write(address, read_unaligned(code.as_ptr() as *const u16));
            }
            3..=4 => {
                let existing_code = read_unaligned(address as *const u32);
                let code = read_bytes_as_u32(code.as_ptr(), num_bytes);
                let mask = match num_bytes {
                    3 => 0x00_FF_FF_FF_u32.to_le(),
                    4 => 0xFF_FF_FF_FF,
                    _ => unreachable_unchecked(),
                };

                let combined_code = (existing_code & !mask) | (code & mask);
                TWriter::atomic_write(address, combined_code);
            }
            5..=8 => {
                let existing_code = read_unaligned(address as *const u64);
                let code: u64 = read_bytes_as_u64(code.as_ptr(), num_bytes);
                let mask: u64 = match num_bytes {
                    5 => 0x00_00_00_FF_FF_FF_FF_FF_u64.to_le(),
                    6 => 0x00_00_FF_FF_FF_FF_FF_FF_u64.to_le(),
                    7 => 0x00_FF_FF_FF_FF_FF_FF_FF_u64.to_le(),
                    8 => 0xFF_FF_FF_FF_FF_FF_FF_FF_u64,
                    _ => unreachable_unchecked(),
                };

                let combined_code = (existing_code & !mask) | (code & mask);
                TWriter::atomic_write(address, combined_code);
            }
            9..=16 => {
                let existing_code = read_unaligned(address as *const u128);
                let code: u128 = read_bytes_as_u128(code.as_ptr(), num_bytes);
                let mask: u128 = match num_bytes {
                    9 => 0x00_00_00_00_00_00_00_FF_FF_FF_FF_FF_FF_FF_FF_FF_u128.to_le(),
                    10 => 0x00_00_00_00_00_00_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_u128.to_le(),
                    11 => 0x00_00_00_00_00_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_u128.to_le(),
                    12 => 0x00_00_00_00_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_u128.to_le(),
                    13 => 0x00_00_00_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_u128.to_le(),
                    14 => 0x00_00_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_u128.to_le(),
                    15 => 0x00_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_u128.to_le(),
                    16 => 0xFF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_FF_u128,
                    _ => unreachable!(),
                };

                let combined_code = (existing_code & !mask) | (code & mask);
                TWriter::atomic_write(address, combined_code);
            }

            _ => panic!("Unsupported num_bytes in atomic_overwrite_with_mask"),
        }
    }
}

unsafe fn read_bytes_as_u32(address: *const u8, num_bytes: usize) -> u32 {
    if num_bytes == 4 {
        read_unaligned(address as *const u32)
    } else {
        // num_bytes is 3.
        if cfg!(target_endian = "little") {
            // Little-endian: LSB is at the lowest address.
            let lower_bytes = read_unaligned(address as *const u16) as u32;
            let upper_byte = *address.add(2) as u32;
            lower_bytes | (upper_byte << 16) // Leftmost byte should be empty.
        } else {
            // Big-endian: MSB is at the lowest address.
            let lower_bytes = read_unaligned(address as *const u16) as u32;
            let upper_byte = *address.add(2) as u32;
            lower_bytes << 16 | (upper_byte << 8) // We shift 8 because the rightmost byte should be empty
        }
    }
}

unsafe fn read_bytes_as_u64(address: *const u8, num_bytes: usize) -> u64 {
    if num_bytes == 8 {
        read_unaligned(address as *const u64)
    } else {
        // num_bytes is between 5 and 7
        let mut value: u64 = read_unaligned(address as *const u32) as u64;
        if cfg!(target_endian = "little") {
            // Little-endian
            // Byte 5
            value |= (*address.add(4) as u64) << 32;

            if num_bytes == 7 {
                // Bytes 6-7
                value |= (read_unaligned(address.add(5) as *const u16) as u64) << 40;
            } else if num_bytes == 6 {
                // Byte 6
                value |= (*address.add(5) as u64) << 40;
            }
        } else {
            // Big-endian
            // Bytes 0-4
            value <<= 32;

            // Byte 5
            value |= (*address.add(4) as u64) << 24;

            if num_bytes == 7 {
                // Bytes 6-7
                value |= (read_unaligned(address.add(5) as *const u16) as u64) << 8;
            } else if num_bytes == 6 {
                // Byte 6
                value |= (*address.add(5) as u64) << 16;
            }
        }

        value
    }
}

unsafe fn read_bytes_as_u128(address: *const u8, num_bytes: usize) -> u128 {
    if num_bytes == 16 {
        return read_unaligned(address as *const u128);
    }

    if cfg!(target_endian = "little") {
        // Read first 8 bytes
        let mut value = read_unaligned(address as *const u64) as u128;

        // Bytes >= 8
        for i in 8..num_bytes {
            value |= (*address.add(i) as u128) << (8 * i);
        }
        value
    } else {
        // Read first 8 bytes
        let mut value = (read_unaligned(address as *const u64) as u128) << 64;

        // Bytes >= 8
        for i in 8..num_bytes {
            value |= (*address.add(i) as u128) << ((15 - i) * 8);
        }

        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! atomic_write_test {
        ($name:ident, $num_bytes:expr, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let mut buffer = [0xFFu8; 16];
                let address = buffer.as_mut_ptr() as usize;

                unsafe {
                    atomic_write_masked::<NativeMemoryAtomicWriter>(address, &$input, $num_bytes);
                }
                let result = &buffer[0..$num_bytes];
                assert_eq!(result, $expected);

                // Assert that the rest of the buffer remains 0xFF
                for &byte in &buffer[$num_bytes..] {
                    assert_eq!(byte, 0xFF);
                }
            }
        };
    }

    atomic_write_test!(test_atomic_write_masked_1_byte, 1, [0xAB], &[0xAB]);
    atomic_write_test!(
        test_atomic_write_masked_2_bytes,
        2,
        [0xCD, 0xEF],
        &[0xCD, 0xEF]
    );
    atomic_write_test!(
        test_atomic_write_masked_3_bytes,
        3,
        [0x01, 0x02, 0x03],
        &[0x01, 0x02, 0x03]
    );
    atomic_write_test!(
        test_atomic_write_masked_4_bytes,
        4,
        [0x04, 0x05, 0x06, 0x07],
        &[0x04, 0x05, 0x06, 0x07]
    );
    atomic_write_test!(
        test_atomic_write_masked_5_bytes,
        5,
        [0x08, 0x09, 0x0A, 0x0B, 0x0C],
        &[0x08, 0x09, 0x0A, 0x0B, 0x0C]
    );
    atomic_write_test!(
        test_atomic_write_masked_6_bytes,
        6,
        [0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12],
        &[0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12]
    );
    atomic_write_test!(
        test_atomic_write_masked_7_bytes,
        7,
        [0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19],
        &[0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19]
    );
    atomic_write_test!(
        test_atomic_write_masked_8_bytes,
        8,
        [0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21],
        &[0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21]
    );
    atomic_write_test!(
        test_atomic_write_masked_9_bytes,
        9,
        [0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A],
        &[0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A]
    );

    atomic_write_test!(
        test_atomic_write_masked_10_bytes,
        10,
        [0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39],
        &[0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39]
    );

    atomic_write_test!(
        test_atomic_write_masked_11_bytes,
        11,
        [0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44],
        &[0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44]
    );

    atomic_write_test!(
        test_atomic_write_masked_12_bytes,
        12,
        [0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50],
        &[0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50]
    );

    atomic_write_test!(
        test_atomic_write_masked_13_bytes,
        13,
        [0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D],
        &[0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D]
    );

    atomic_write_test!(
        test_atomic_write_masked_14_bytes,
        14,
        [0x5E, 0x5F, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B],
        &[0x5E, 0x5F, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B]
    );

    atomic_write_test!(
        test_atomic_write_masked_15_bytes,
        15,
        [
            0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
            0x7A
        ],
        &[
            0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
            0x7A
        ]
    );

    atomic_write_test!(
        test_atomic_write_masked_16_bytes,
        16,
        [
            0x7B, 0x7C, 0x7D, 0x7E, 0x7F, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88,
            0x89, 0x8A
        ],
        &[
            0x7B, 0x7C, 0x7D, 0x7E, 0x7F, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88,
            0x89, 0x8A
        ]
    );
}
