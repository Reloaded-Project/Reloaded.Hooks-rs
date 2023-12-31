extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::{X86jitError, ARCH_NOT_SUPPORTED};
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::CallAbs};

pub(crate) fn encode_call_absolute(
    a: &mut CodeAssembler,
    x: &CallAbs<AllRegisters>,
) -> Result<(), X86jitError<AllRegisters>> {
    if a.bitness() == 64 && cfg!(feature = "x64") {
        #[cfg(feature = "x64")]
        {
            let target_reg = x.scratch_register.as_iced_64()?;
            a.mov(target_reg, x.target_address as u64)?;
            a.call(target_reg)?;
        }
    } else if a.bitness() == 32 && cfg!(feature = "x86") {
        let target_reg = x.scratch_register.as_iced_32()?;
        a.mov(target_reg, x.target_address as u32)?;
        a.call(target_reg)?;
    } else {
        return Err(JitError::ThirdPartyAssemblerError(ARCH_NOT_SUPPORTED.to_string()).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};

    use crate::{
        x64::{self, jit::JitX64},
        x86::{self, jit::JitX86},
    };

    #[test]
    fn call_absolute_x86() {
        let operations = vec![Op::CallAbsolute(CallAbs {
            scratch_register: x86::Register::eax,
            target_address: 0x12345678,
        })];
        let result = JitX86::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("b878563412ffd0", hex::encode(result.unwrap()));
    }

    #[test]
    fn call_absolute_x64() {
        let operations = vec![Op::CallAbsolute(CallAbs {
            scratch_register: x64::Register::rax,
            target_address: 0x12345678,
        })];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("48b87856341200000000ffd0", hex::encode(result.unwrap()));
    }
}
