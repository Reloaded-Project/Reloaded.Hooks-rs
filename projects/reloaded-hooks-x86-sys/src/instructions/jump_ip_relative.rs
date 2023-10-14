extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::operation_aliases::JumpIpRel;

use crate::{all_registers::AllRegisters, jit_common::convert_error};
use iced_x86::code_asm::qword_ptr;

pub(crate) fn encode_jump_ip_relative(
    a: &mut CodeAssembler,
    x: &JumpIpRel<AllRegisters>,
    address: usize,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 32 {
        return Err(JitError::ThirdPartyAssemblerError(
            "Jump IP Relative is only Supported on 64-bit!".to_string(),
        ));
    }

    let isns = a.instructions();
    let current_ip = if !isns.is_empty() {
        isns.last().unwrap().next_ip()
    } else {
        address as u64
    };

    let relative_offset = x.target_address.wrapping_sub(current_ip as usize);
    a.jmp(qword_ptr(iced_x86::Register::RIP) + relative_offset as i32)
        .map_err(convert_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::x64::jit::JitX64;
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};

    #[test]
    fn jmp_rip_relative_x64() {
        let mut jit = JitX64 {};

        let operations = vec![Op::JumpIpRelative(JumpIpRel::new(0x16))];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("ff2510000000", hex::encode(result.unwrap()));
    }

    #[test]
    fn jmp_rip_relative_backwards_x64() {
        let mut jit = JitX64 {};

        let operations = vec![Op::JumpIpRelative(JumpIpRel::new(16))];
        let result = jit.compile(20, &operations);
        assert!(result.is_ok());
        assert_eq!("ff25e2ffffff", hex::encode(result.unwrap()));
    }

    #[test]
    fn jmp_rip_relative_backwards_two_instructions_x64() {
        let mut jit = JitX64 {};

        let operations = vec![
            Op::StackAlloc(StackAlloc::new(10)),
            Op::JumpIpRelative(JumpIpRel::new(16)),
        ];
        let result = jit.compile(20, &operations);
        assert!(result.is_ok());
        assert_eq!(
            "4883ec0aff25f2ffffff",
            hex::encode(result.as_ref().unwrap())
        );
    }
}
