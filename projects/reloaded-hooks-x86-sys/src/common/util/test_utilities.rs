pub fn str_to_vec(hex: String) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|chunk| {
            let hex_str = std::str::from_utf8(chunk).unwrap();
            u8::from_str_radix(hex_str, 16).unwrap()
        })
        .collect()
}

use core::{mem::size_of_val, slice};

pub fn instruction_buffer_as_hex(buf: &[u8]) -> String {
    hex::encode(buf)
}

pub fn instruction_buffer_as_hex_u8(buf: &[u8]) -> String {
    let ptr = buf.as_ptr();
    unsafe {
        let as_u8 = slice::from_raw_parts(ptr, size_of_val(buf));
        hex::encode(as_u8)
    }
}

pub fn assert_encode(expected_hex: &str, buf: &[u8], pc: usize) {
    assert_eq!(expected_hex, instruction_buffer_as_hex(buf));
    assert_eq!(buf.len() * size_of_val(&buf[0]), pc);
}

pub fn assert_encode_with_initial_pc(expected_hex: &str, buf: &[u8], initial_pc: usize, pc: usize) {
    assert_encode(expected_hex, buf, pc - initial_pc);
}

/// Macro to assert a specific type of error result from a function call.
///
/// This macro helps in reducing boilerplate code in tests when checking for specific error types.
///
/// # Parameters
///
/// - `$result`: The `Result` object returned from a function call.
/// - `$expected_error`: The expected error pattern. This should match the error variant you are expecting.
/// - `$pc`: The program counter value to assert against. Typically used to check if the program counter remains unchanged in the case of an error.
/// - `$buf`: The buffer to check the length against. Typically used to check if the buffer remains unchanged in the case of an error.
///
/// # Usage
///
/// ```rust
/// use crate::assert_error;
///
/// fn function_that_errors() -> Result<(), JitError> {
///     Err(JitError::OperandOutOfRange("error".to_string()))
/// }
///
/// fn main() {
///     let result = function_that_errors();
///     let pc = 0;
///     let buf = Vec::new();
///
///     assert_error!(result, JitError::OperandOutOfRange(_), pc, buf);  // Basic usage
/// }
/// ```
///
/// # Overloaded Usage
///
/// The macro can also be used with additional parameters to assert against specific values of program counter and buffer length.
///
/// ```rust
/// use crate::assert_error;
///
/// fn function_that_errors() -> Result<(), JitError> {
///     Err(JitError::OperandOutOfRange("error".to_string()))
/// }
///
/// fn main() {
///     let result = function_that_errors();
///     let pc = 1;
///     let buf = vec![1, 2, 3];
///
///     assert_error!(result, JitError::OperandOutOfRange(_), 1, 3, pc, buf);  // Overloaded usage
/// }
/// ```
#[macro_export]
macro_rules! assert_error {
    ($result:expr, $expected_error:pat, $pc:expr, $buf:expr) => {
        assert!($result.is_err());
        assert!(matches!($result.unwrap_err(), $expected_error));
        assert_eq!(0, $pc);
        assert_eq!(0, $buf.len());
    };
    ($result:expr, $expected_error:pat, $expected_pc:expr, $expected_buf_len:expr, $pc:expr, $buf:expr) => {
        assert!($result.is_err());
        assert!(matches!($result.unwrap_err(), $expected_error));
        assert_eq!($expected_pc, $pc);
        assert_eq!($expected_buf_len, $buf.len());
    };
}
