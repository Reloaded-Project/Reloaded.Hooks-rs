pub fn str_to_vec(hex: String) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|chunk| {
            let hex_str = std::str::from_utf8(chunk).unwrap();
            u8::from_str_radix(hex_str, 16).unwrap()
        })
        .collect()
}

use super::get_stolen_instructions::ZydisInstruction;
use core::{mem::size_of_val, ops::Sub};
use reloaded_hooks_portable::api::{
    rewriter::code_rewriter::{CodeRewriter, CodeRewriterError},
    traits::register_info::RegisterInfo,
};
use zydis::{Decoder, MachineMode, StackWidth, VisibleOperands};

pub fn instruction_buffer_as_hex(buf: &[u8]) -> String {
    hex::encode(buf)
}

pub fn assert_encode(expected_hex: &str, buf: &[u8], pc: usize) {
    assert_eq!(expected_hex, instruction_buffer_as_hex(buf));
    assert_eq!(buf.len() * size_of_val(&buf[0]), pc);
}

pub fn assert_encode_with_initial_pc(expected_hex: &str, buf: &[u8], initial_pc: usize, pc: usize) {
    assert_encode(expected_hex, buf, pc.wrapping_sub(initial_pc));
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

pub(crate) fn test_relocate_instruction<TRegister>(
    instructions: String,
    old_address: usize,
    new_address: usize,
    expected: String,
    scratch_reg: Option<TRegister>,
    is_64bit: bool,
    relocate_func: fn(
        &ZydisInstruction,
        &[u8],
        &mut usize,
        &mut usize,
        Option<TRegister>,
        &mut Vec<u8>,
    ) -> Result<(), CodeRewriterError>,
) {
    // Remove spaces and convert the string to a vector of bytes
    let dec = Decoder::new(
        if is_64bit & cfg!(feature = "x64") {
            MachineMode::LONG_64
        } else if cfg!(feature = "x86") {
            MachineMode::LONG_COMPAT_32
        } else {
            unreachable!();
        },
        if is_64bit & cfg!(feature = "x64") {
            StackWidth::_64
        } else if cfg!(feature = "x86") {
            StackWidth::_32
        } else {
            unreachable!();
        },
    )
    .unwrap();

    let hex_bytes: Vec<u8> = str_to_vec(instructions);
    let ins = dec
        .decode_all::<VisibleOperands>(&hex_bytes, old_address as u64)
        .next()
        .unwrap()
        .unwrap();

    let mut pc = old_address;
    let mut dest_address = new_address;
    let mut result = Vec::new();
    relocate_func(
        &ins.2,
        ins.1,
        &mut dest_address,
        &mut pc,
        scratch_reg,
        &mut result,
    )
    .unwrap();

    // Verify we encoded the correct data.
    assert_eq!(hex::encode(&result), expected);

    // Check we have advanced destination correctly.
    assert_eq!(result.len(), dest_address.sub(new_address));

    // Check we have advanced PC correctly.
    assert_eq!(hex_bytes.len(), pc.sub(old_address));
}

pub(crate) fn test_rewrite_code_with_buffer<
    TCodeRewriter: CodeRewriter<TRegister>,
    TRegister: RegisterInfo,
>(
    instructions: String,
    old_address: usize,
    new_address: usize,
    expected: String,
    scratch_reg: Option<TRegister>,
) {
    // Convert the string of instructions to a vector of bytes
    let hex_bytes: Vec<u8> = str_to_vec(instructions);
    let old_code_size = hex_bytes.len();

    // Create a buffer for the rewritten code
    let mut existing_buffer: Vec<u8> = Vec::new();

    // Call rewrite_code_with_buffer
    let result = unsafe {
        TCodeRewriter::rewrite_code_with_buffer(
            hex_bytes.as_ptr(),
            old_code_size,
            old_address,
            new_address,
            scratch_reg,
            &mut existing_buffer,
        )
    };

    // Assert that the operation was successful
    assert!(result.is_ok());

    // Verify the rewritten code matches the expected output
    assert_eq!(hex::encode(&existing_buffer), expected);

    // Additional assertions can be added if necessary
}
