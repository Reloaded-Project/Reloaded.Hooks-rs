extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::X86jitError;
use alloc::string::ToString;
use iced_x86::code_asm::{qword_ptr, CodeAssembler};
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::CallIpRel};

#[cfg(feature = "x64")]
pub(crate) fn encode_call_ip_relative(
    a: &mut CodeAssembler,
    x: &CallIpRel<AllRegisters>,
    address: usize,
) -> Result<(), X86jitError<AllRegisters>> {
    if a.bitness() == 32 {
        return Err(JitError::ThirdPartyAssemblerError(
            "Call IP Relative is only Supported on 64-bit!".to_string(),
        )
        .into());
    }

    let isns = a.instructions();
    let current_ip = if !isns.is_empty() {
        isns.last().unwrap().next_ip()
    } else {
        address as u64
    };

    let relative_offset = x.target_address.wrapping_sub(current_ip as usize);
    a.call(qword_ptr(iced_x86::Register::RIP) + relative_offset as i32)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::x64::jit::JitX64;
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};

    #[test]
    fn call_rip_relative_x64() {
        let operations = vec![Op::CallIpRelative(CallIpRel::new(0x16))];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("ff1510000000", hex::encode(result.unwrap()));
    }

    #[test]
    fn call_rip_relative_backwards_x64() {
        let operations = vec![Op::CallIpRelative(CallIpRel::new(16))];
        let result = JitX64::compile(20, &operations);
        assert!(result.is_ok());
        assert_eq!("ff15e2ffffff", hex::encode(result.as_ref().unwrap()))
    }

    #[test]
    fn call_rip_relative_two_instructions_x64() {
        let operations = vec![
            Op::StackAlloc(StackAlloc::new(10)),
            Op::CallIpRelative(CallIpRel::new(16)),
        ];
        let result = JitX64::compile(20, &operations);
        assert!(result.is_ok());
        assert_eq!("4883ec0aff15f2ffffff", hex::encode(result.unwrap()));
    }
}
