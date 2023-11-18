extern crate alloc;

use alloc::string::ToString;
use iced_x86::Code;
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

/// Inverts the branch condition provided.
/// This is used to invert the branch condition of the original code.
/// For example, if the original code was `je`, then the inverted code would be `jne`.
///
/// # Parameters
/// * `code`: The code to invert.
pub(crate) fn invert_branch_condition(
    code: iced_x86::Code,
) -> Result<iced_x86::Code, CodeRewriterError> {
    match code {
        Code::Jo_rel8_16 => Ok(Code::Jno_rel8_16),
        Code::Jo_rel8_32 => Ok(Code::Jno_rel8_32),
        Code::Jo_rel8_64 => Ok(Code::Jno_rel8_64),
        Code::Jno_rel8_16 => Ok(Code::Jo_rel8_16),
        Code::Jno_rel8_32 => Ok(Code::Jo_rel8_32),
        Code::Jno_rel8_64 => Ok(Code::Jo_rel8_64),

        Code::Jb_rel8_16 => Ok(Code::Jae_rel8_16),
        Code::Jb_rel8_32 => Ok(Code::Jae_rel8_32),
        Code::Jb_rel8_64 => Ok(Code::Jae_rel8_64),
        Code::Jae_rel8_16 => Ok(Code::Jb_rel8_16),
        Code::Jae_rel8_32 => Ok(Code::Jb_rel8_32),
        Code::Jae_rel8_64 => Ok(Code::Jb_rel8_64),

        Code::Je_rel8_16 => Ok(Code::Jne_rel8_16),
        Code::Je_rel8_32 => Ok(Code::Jne_rel8_32),
        Code::Je_rel8_64 => Ok(Code::Jne_rel8_64),
        Code::Jne_rel8_16 => Ok(Code::Je_rel8_16),
        Code::Jne_rel8_32 => Ok(Code::Je_rel8_32),
        Code::Jne_rel8_64 => Ok(Code::Je_rel8_64),

        Code::Jbe_rel8_16 => Ok(Code::Ja_rel8_16),
        Code::Jbe_rel8_32 => Ok(Code::Ja_rel8_32),
        Code::Jbe_rel8_64 => Ok(Code::Ja_rel8_64),
        Code::Ja_rel8_16 => Ok(Code::Jbe_rel8_16),
        Code::Ja_rel8_32 => Ok(Code::Jbe_rel8_32),
        Code::Ja_rel8_64 => Ok(Code::Jbe_rel8_64),

        Code::Js_rel8_16 => Ok(Code::Jns_rel8_16),
        Code::Js_rel8_32 => Ok(Code::Jns_rel8_32),
        Code::Js_rel8_64 => Ok(Code::Jns_rel8_64),
        Code::Jns_rel8_16 => Ok(Code::Js_rel8_16),
        Code::Jns_rel8_32 => Ok(Code::Js_rel8_32),
        Code::Jns_rel8_64 => Ok(Code::Js_rel8_64),

        Code::Jp_rel8_16 => Ok(Code::Jnp_rel8_16),
        Code::Jp_rel8_32 => Ok(Code::Jnp_rel8_32),
        Code::Jp_rel8_64 => Ok(Code::Jnp_rel8_64),
        Code::Jnp_rel8_16 => Ok(Code::Jp_rel8_16),
        Code::Jnp_rel8_32 => Ok(Code::Jp_rel8_32),
        Code::Jnp_rel8_64 => Ok(Code::Jp_rel8_64),

        Code::Jl_rel8_16 => Ok(Code::Jge_rel8_16),
        Code::Jl_rel8_32 => Ok(Code::Jge_rel8_32),
        Code::Jl_rel8_64 => Ok(Code::Jge_rel8_64),
        Code::Jge_rel8_16 => Ok(Code::Jl_rel8_16),
        Code::Jge_rel8_32 => Ok(Code::Jl_rel8_32),
        Code::Jge_rel8_64 => Ok(Code::Jl_rel8_64),

        Code::Jle_rel8_16 => Ok(Code::Jg_rel8_16),
        Code::Jle_rel8_32 => Ok(Code::Jg_rel8_32),
        Code::Jle_rel8_64 => Ok(Code::Jg_rel8_64),
        Code::Jg_rel8_16 => Ok(Code::Jle_rel8_16),
        Code::Jg_rel8_32 => Ok(Code::Jle_rel8_32),
        Code::Jg_rel8_64 => Ok(Code::Jle_rel8_64),

        Code::Jo_rel16 => Ok(Code::Jno_rel16),
        Code::Jo_rel32_32 => Ok(Code::Jno_rel32_32),
        Code::Jo_rel32_64 => Ok(Code::Jno_rel32_64),
        Code::Jno_rel16 => Ok(Code::Jo_rel16),
        Code::Jno_rel32_32 => Ok(Code::Jo_rel32_32),
        Code::Jno_rel32_64 => Ok(Code::Jo_rel32_64),

        Code::Jb_rel16 => Ok(Code::Jae_rel16),
        Code::Jb_rel32_32 => Ok(Code::Jae_rel32_32),
        Code::Jb_rel32_64 => Ok(Code::Jae_rel32_64),
        Code::Jae_rel16 => Ok(Code::Jb_rel16),
        Code::Jae_rel32_32 => Ok(Code::Jb_rel32_32),
        Code::Jae_rel32_64 => Ok(Code::Jb_rel32_64),

        Code::Je_rel16 => Ok(Code::Jne_rel16),
        Code::Je_rel32_32 => Ok(Code::Jne_rel32_32),
        Code::Je_rel32_64 => Ok(Code::Jne_rel32_64),
        Code::Jne_rel16 => Ok(Code::Je_rel16),
        Code::Jne_rel32_32 => Ok(Code::Je_rel32_32),
        Code::Jne_rel32_64 => Ok(Code::Je_rel32_64),

        Code::Jbe_rel16 => Ok(Code::Ja_rel16),
        Code::Jbe_rel32_32 => Ok(Code::Ja_rel32_32),
        Code::Jbe_rel32_64 => Ok(Code::Ja_rel32_64),
        Code::Ja_rel16 => Ok(Code::Jbe_rel16),
        Code::Ja_rel32_32 => Ok(Code::Jbe_rel32_32),
        Code::Ja_rel32_64 => Ok(Code::Jbe_rel32_64),

        Code::Js_rel16 => Ok(Code::Jns_rel16),
        Code::Js_rel32_32 => Ok(Code::Jns_rel32_32),
        Code::Js_rel32_64 => Ok(Code::Jns_rel32_64),
        Code::Jns_rel16 => Ok(Code::Js_rel16),
        Code::Jns_rel32_32 => Ok(Code::Js_rel32_32),
        Code::Jns_rel32_64 => Ok(Code::Js_rel32_64),

        Code::Jp_rel16 => Ok(Code::Jnp_rel16),
        Code::Jp_rel32_32 => Ok(Code::Jnp_rel32_32),
        Code::Jp_rel32_64 => Ok(Code::Jnp_rel32_64),
        Code::Jnp_rel16 => Ok(Code::Jp_rel16),
        Code::Jnp_rel32_32 => Ok(Code::Jp_rel32_32),
        Code::Jnp_rel32_64 => Ok(Code::Jp_rel32_64),

        Code::Jl_rel16 => Ok(Code::Jge_rel16),
        Code::Jl_rel32_32 => Ok(Code::Jge_rel32_32),
        Code::Jl_rel32_64 => Ok(Code::Jge_rel32_64),
        Code::Jge_rel16 => Ok(Code::Jl_rel16),
        Code::Jge_rel32_32 => Ok(Code::Jl_rel32_32),
        Code::Jge_rel32_64 => Ok(Code::Jl_rel32_64),

        Code::Jle_rel16 => Ok(Code::Jg_rel16),
        Code::Jle_rel32_32 => Ok(Code::Jg_rel32_32),
        Code::Jle_rel32_64 => Ok(Code::Jg_rel32_64),
        Code::Jg_rel16 => Ok(Code::Jle_rel16),
        Code::Jg_rel32_32 => Ok(Code::Jle_rel32_32),
        Code::Jg_rel32_64 => Ok(Code::Jle_rel32_64),
        _ => Err(CodeRewriterError::ThirdPartyAssemblerError(
            "Sewer Skill Issue or Updated Instruction Set. Failed to invert branch.".to_string(),
        )),
    }
}
