use crate::api::buffers::buffer_abstractions::Buffer;
use core::ptr::read_unaligned;

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

/// Overwrites bytes at the specified address with provided bytes using atomic operations.
/// This will only replace the specified number of bytes, preserving the rest.
#[inline]
pub fn atomic_write_masked<TWriter>(address: usize, code: &[u8], num_bytes: usize)
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
                let code = read_unaligned(code.as_ptr() as *const u32);
                let mask = match num_bytes {
                    3 => 0x00_FF_FF_FF_u32.to_le(),
                    4 => 0xFF_FF_FF_FF,
                    _ => unreachable!(),
                };

                let combined_code = (existing_code & !mask) | (code & mask);
                TWriter::atomic_write(address, combined_code);
            }
            5..=8 => {
                let existing_code = read_unaligned(address as *const u64);
                let mut temp_code: u64 = 0;
                let mask: u64 = match num_bytes {
                    5 => 0x00_00_00_FF_FF_FF_FF_FF_u64.to_le(),
                    6 => 0x00_00_FF_FF_FF_FF_FF_FF_u64.to_le(),
                    7 => 0x00_FF_FF_FF_FF_FF_FF_FF_u64.to_le(),
                    8 => 0xFF_FF_FF_FF_FF_FF_FF_FF_u64,
                    _ => unreachable!(),
                };

                if cfg!(target_endian = "little") {
                    for (i, &byte) in code.iter().enumerate() {
                        temp_code |= (byte as u64) << (i * 8);
                    }
                } else {
                    // Big-endian case
                    for (i, &byte) in code.iter().enumerate() {
                        temp_code |= (byte as u64) << ((7 - i) * 8);
                    }
                }

                let combined_code = (existing_code & !mask) | (temp_code & mask);
                TWriter::atomic_write(address, combined_code);
            }
            _ => panic!("Unsupported num_bytes in atomic_overwrite_with_mask"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! atomic_write_test {
        ($name:ident, $num_bytes:expr, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let mut buffer = [0xFFu8; 8];
                let address = buffer.as_mut_ptr() as usize;

                atomic_write_masked::<MockAtomicWriter>(address, &$input, $num_bytes);
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

    struct MockAtomicWriter {}

    impl AtomicWriter for MockAtomicWriter {
        fn atomic_write<TInteger>(address: usize, value: TInteger)
        where
            Self: Sized,
        {
            unsafe {
                *(address as *mut TInteger) = value;
            }
        }
    }
}
