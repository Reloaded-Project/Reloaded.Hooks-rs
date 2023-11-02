use core::{mem::size_of_val, slice};

use crate::code_rewriter::instruction_rewrite_result::InstructionRewriteResult;

pub trait ToHexString {
    fn to_hex_string(&self) -> String;
}

impl ToHexString for InstructionRewriteResult {
    fn to_hex_string(&self) -> String {
        let mut buf = Vec::new();
        self.append_to_buffer(&mut buf);
        let i32 = unsafe { core::mem::transmute::<Vec<u32>, Vec<i32>>(buf) };
        instruction_buffer_as_hex(&i32)
    }
}

impl ToHexString for u32 {
    fn to_hex_string(&self) -> String {
        let buf = vec![*self as i32];
        instruction_buffer_as_hex(&buf)
    }
}

pub fn instruction_buffer_as_hex(buf: &[i32]) -> String {
    let ptr = buf.as_ptr() as *const u8;
    unsafe {
        let as_u8 = slice::from_raw_parts(ptr, size_of_val(buf));
        hex::encode(as_u8)
    }
}

pub fn assert_encode(expected_hex: &str, buf: &[i32], pc: usize) {
    assert_eq!(expected_hex, instruction_buffer_as_hex(buf));
    assert_eq!(buf.len() * size_of_val(&buf[0]), pc);
}

pub fn assert_encode_with_initial_pc(
    expected_hex: &str,
    buf: &[i32],
    initial_pc: usize,
    pc: usize,
) {
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
