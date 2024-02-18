extern crate alloc;

use alloc::string::ToString;
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;
use smallvec::SmallVec;
use zydis::{MachineMode, StackWidth, VisibleOperands};

pub type ZydisInstruction = zydis::Instruction<VisibleOperands>;

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
    let mut dec = zydis::Decoder::new(
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
    .map_err(|e| CodeRewriterError::ThirdPartyAssemblerError(e.description().into()))?;

    get_stolen_instructions_length_from_decoder(&mut dec, code, min_bytes, ip)
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
    decoder: &mut zydis::Decoder,
    code: &[u8],
    min_bytes: u8,
    ip: usize,
) -> Result<(u32, u32), CodeRewriterError> {
    let required_bytes = min_bytes as u32;
    let mut total_bytes: u32 = 0;
    let mut total_instructions: u32 = 0;

    for dec in decoder.decode_all::<VisibleOperands>(code, ip as u64) {
        // ip, insn_bytes, isn
        let ins = dec.map_err(|_| {
            CodeRewriterError::FailedToDisasm(
                total_bytes.to_string(),
                hex::encode(&code[total_bytes as usize..]),
            )
        })?;

        total_bytes += ins.1.len() as u32;
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
