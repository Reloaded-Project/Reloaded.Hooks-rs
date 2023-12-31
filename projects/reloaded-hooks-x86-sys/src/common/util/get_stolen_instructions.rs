extern crate alloc;

use alloc::string::ToString;
use iced_x86::{Decoder, DecoderOptions, Instruction};
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;
use smallvec::SmallVec;

/// Retrieves the 'stolen' instructions from the provided code region.
/// The 'stolen' instructions represent a minimum amount of code that needs to be
/// copied out of the original code region in order to be able to place a jump
/// to our hook function.
///
/// # Parameters
/// `is_64bit`: Whether the code is 64bit or not.
/// `min_bytes`: The minimum amount of bytes to copy.
/// `code`: The code region to copy from.
/// `ip`: The instruction pointer corresponding to first instruction in 'code'.
///
/// # Returns
/// Either a re-encode error, in which case the operation fails, or a
/// list of decoded instructions and their combined length.
pub(crate) fn get_stolen_instructions(
    is_64bit: bool,
    min_bytes: usize,
    code: &[u8],
    ip: usize,
) -> Result<(SmallVec<[Instruction; 4]>, u32), CodeRewriterError> {
    let mut decoder = Decoder::with_ip(
        if is_64bit & cfg!(feature = "x64") {
            64
        } else if cfg!(feature = "x86") {
            32
        } else {
            0
        },
        code,
        ip as u64,
        DecoderOptions::NO_INVALID_CHECK,
    );

    get_stolen_instructions_from_decoder(&mut decoder, code, min_bytes)
}

/// Retrieves the 'stolen' instructions from the provided code region.
/// The 'stolen' instructions represent a minimum amount of code that needs to be
/// copied out of the original code region in order to be able to place a jump
/// to our hook function.
///
/// # Parameters
/// `decoder`: The decoder responsible for decoding operation.
/// `code`: The code region to copy from.
/// `min_bytes`: The minimum amount of bytes to copy.
///
/// # Returns
/// Either a re-encode error, in which case the operation fails, or a
/// list of decooded instructions and their combined length.
pub(crate) fn get_stolen_instructions_from_decoder(
    decoder: &mut Decoder,
    code: &[u8],
    min_bytes: usize,
) -> Result<(SmallVec<[Instruction; 4]>, u32), CodeRewriterError> {
    let required_bytes = min_bytes as u32;
    let mut total_bytes: u32 = 0;
    let mut orig_instructions: SmallVec<[Instruction; 4]> = Default::default();

    let mut instr = Instruction::default();
    while decoder.can_decode() {
        decoder.decode_out(&mut instr);

        if instr.is_invalid() {
            return Err(CodeRewriterError::FailedToDisasm(
                total_bytes.to_string(),
                hex::encode(&code[total_bytes as usize..]),
            ));
        }

        orig_instructions.push(instr);
        total_bytes += instr.len() as u32;
        if total_bytes >= required_bytes {
            break;
        }
    }

    if total_bytes < required_bytes {
        return Err(CodeRewriterError::InsufficientBytes);
    }

    debug_assert!(!orig_instructions.is_empty());
    Ok((orig_instructions, total_bytes))
}

/// Retrieves the length of 'stolen' instructions from the provided code region.
/// The length represents the minimum amount of code that needs to be
/// copied to place a jump to our hook function.
///
/// # Parameters
/// `is_64bit`: Whether the code is 64bit or not.
/// `min_bytes`: The minimum amount of bytes to copy.
/// `code`: The code region to copy from.
/// `ip`: The instruction pointer corresponding to the first instruction in 'code'.
///
/// # Returns
/// Either a tuple (ins_length_bytes, num_instructions) error or the length of the decoded instructions.
pub(crate) fn get_stolen_instructions_lengths(
    is_64bit: bool,
    min_bytes: u8,
    code: &[u8],
    ip: usize,
) -> Result<(u32, u32), CodeRewriterError> {
    let mut decoder = Decoder::with_ip(
        if is_64bit & cfg!(feature = "x64") {
            64
        } else if cfg!(feature = "x86") {
            32
        } else {
            0
        },
        code,
        ip as u64,
        DecoderOptions::NONE,
    );

    get_stolen_instructions_length_from_decoder(&mut decoder, code, min_bytes)
}

/// Retrieves the length of 'stolen' instructions using the provided decoder.
/// The length represents the minimum amount of code that needs to be
/// copied to place a jump to our hook function.
///
/// # Parameters
/// `decoder`: The decoder responsible for decoding operations.
/// `code`: The code region to copy from.
/// `min_bytes`: The minimum amount of bytes to copy.
///
/// # Returns
/// Either a tuple (ins_length_bytes, num_instructions) error or the length of the decoded instructions.
pub(crate) fn get_stolen_instructions_length_from_decoder(
    decoder: &mut Decoder,
    code: &[u8],
    min_bytes: u8,
) -> Result<(u32, u32), CodeRewriterError> {
    let required_bytes = min_bytes as u32;
    let mut total_bytes: u32 = 0;
    let mut total_instructions: u32 = 0;

    let mut instr = Instruction::default();
    while decoder.can_decode() {
        decoder.decode_out(&mut instr);

        if instr.is_invalid() {
            return Err(CodeRewriterError::FailedToDisasm(
                total_bytes.to_string(),
                hex::encode(&code[total_bytes as usize..]),
            ));
        }

        total_bytes += instr.len() as u32;
        total_instructions += 1;
        if total_bytes >= required_bytes {
            break;
        }
    }

    if total_bytes < required_bytes {
        return Err(CodeRewriterError::InsufficientBytes);
    }

    Ok((total_bytes, total_instructions))
}
