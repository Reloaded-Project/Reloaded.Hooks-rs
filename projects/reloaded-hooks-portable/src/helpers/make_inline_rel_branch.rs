use crate::api::errors::inline_branch_error::InlineBranchError;
use core::mem::{transmute, MaybeUninit};

// x86 = 2/4/5 bytes
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub const INLINE_BRANCH_LEN: usize = 5;

// ARM64 = 4 bytes
// ARM = 4 bytes
// MIPS = 4 bytes
// PPC = 4 bytes
// RISC-V = 4 bytes
// SPARC = 4 bytes
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub const INLINE_BRANCH_LEN: usize = 4;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const X86_REL8_BRANCH: usize = 2;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const X86_REL16_BRANCH: usize = 4; // with prefix

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const X86_REL32_BRANCH: usize = 5;

#[cfg(target_arch = "aarch64")]
const AARCH64_BRANCH: usize = 4;

pub fn make_inline_branch(rc: &[u8]) -> Result<[u8; INLINE_BRANCH_LEN], InlineBranchError> {
    #[cfg(target_arch = "aarch64")]
    return make_inline_branch_aarch64(rc);

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    return make_inline_branch_x86(rc);

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
    return make_inline_branch_fallback(rc);
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
fn make_inline_branch_fallback(rc: &[u8]) -> Result<[u8; INLINE_BRANCH_LEN], InlineBranchError> {
    if rc.len() <= INLINE_BRANCH_LEN {
        // Copy the available bytes (up to 5) from the rc slice.
        // This has 0 overhead, damn, Rust is quite magical
        let mut result = [0; INLINE_BRANCH_LEN];
        for (dest, &src) in result.iter_mut().zip(rc.iter()) {
            *dest = src;
        }

        return Ok(result);
    }

    Err(InlineBranchError::ArrayTooShort(
        INLINE_BRANCH_LEN,
        rc.len(),
    ))
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn make_inline_branch_x86(rc: &[u8]) -> Result<[u8; INLINE_BRANCH_LEN], InlineBranchError> {
    unsafe {
        match rc.len() {
            X86_REL8_BRANCH => copy_rel8_branch(rc),
            X86_REL32_BRANCH => copy_rel32_branch(rc),
            X86_REL16_BRANCH => copy_rel16_branch_cold(rc),
            _ => unexpected_array_length_cold(rc.len()),
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn copy_rel8_branch(rc: &[u8]) -> Result<[u8; INLINE_BRANCH_LEN], InlineBranchError> {
    let mut result: [MaybeUninit<u8>; INLINE_BRANCH_LEN] =
        unsafe { MaybeUninit::uninit().assume_init() };
    result[..X86_REL8_BRANCH].copy_from_slice(transmute(&rc[..X86_REL8_BRANCH]));
    Ok(transmute(result))
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn copy_rel32_branch(rc: &[u8]) -> Result<[u8; INLINE_BRANCH_LEN], InlineBranchError> {
    let mut result: [MaybeUninit<u8>; INLINE_BRANCH_LEN] =
        unsafe { MaybeUninit::uninit().assume_init() };
    result[..X86_REL32_BRANCH].copy_from_slice(transmute(&rc[..X86_REL32_BRANCH]));
    Ok(transmute(result))
}

#[cold]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn copy_rel16_branch_cold(rc: &[u8]) -> Result<[u8; INLINE_BRANCH_LEN], InlineBranchError> {
    let mut result: [MaybeUninit<u8>; INLINE_BRANCH_LEN] =
        unsafe { MaybeUninit::uninit().assume_init() };
    result[..X86_REL16_BRANCH].copy_from_slice(transmute(&rc[..X86_REL16_BRANCH]));
    Ok(transmute(result))
}

#[cold]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn unexpected_array_length_cold(len: usize) -> Result<[u8; INLINE_BRANCH_LEN], InlineBranchError> {
    Err(InlineBranchError::ArrayTooShort(INLINE_BRANCH_LEN, len))
}

#[cfg(target_arch = "aarch64")]
fn make_inline_branch_aarch64(rc: &[u8]) -> Result<[u8; INLINE_BRANCH_LEN], InlineBranchError> {
    if rc.len() == AARCH64_BRANCH {
        unsafe {
            let mut result: [MaybeUninit<u8>; INLINE_BRANCH_LEN] =
                MaybeUninit::uninit().assume_init();
            result[..AARCH64_BRANCH].copy_from_slice(transmute(&rc[..AARCH64_BRANCH]));
            return Ok(transmute(result));
        }
    }

    Err(InlineBranchError::UnexpectedArrayLength(rc.len()))
}
