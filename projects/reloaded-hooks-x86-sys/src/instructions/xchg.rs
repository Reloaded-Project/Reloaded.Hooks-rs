extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::XChg};

use crate::all_registers::AllRegisters;
use crate::common::jit_common::convert_error;

macro_rules! encode_xchg_vector {
    ($fn_name:ident, $reg_type:ident, $mov_instr:ident) => {
        fn $fn_name(
            a: &mut CodeAssembler,
            xchg: &XChg<AllRegisters>,
        ) -> Result<(), JitError<AllRegisters>> {
            let scratch = get_scratch(xchg.scratch)?;

            a.$mov_instr(scratch.$reg_type()?, xchg.register1.$reg_type()?)
                .map_err(convert_error)?;
            a.$mov_instr(xchg.register1.$reg_type()?, xchg.register2.$reg_type()?)
                .map_err(convert_error)?;
            a.$mov_instr(xchg.register2.$reg_type()?, scratch.$reg_type()?)
                .map_err(convert_error)?;
            Ok(())
        }
    };
}

encode_xchg_vector!(encode_xchg_xmm, as_iced_xmm, movaps);
encode_xchg_vector!(encode_xchg_ymm, as_iced_ymm, vmovaps);
encode_xchg_vector!(encode_xchg_zmm, as_iced_zmm, vmovaps);

pub(crate) fn encode_xchg(
    a: &mut CodeAssembler,
    xchg: &XChg<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if xchg.register1.is_32() && xchg.register2.is_32() {
        a.xchg(xchg.register1.as_iced_32()?, xchg.register2.as_iced_32()?)
            .map_err(convert_error)?
    } else if xchg.register1.is_64() && xchg.register2.is_64() && cfg!(feature = "x64") {
        #[cfg(feature = "x64")]
        a.xchg(xchg.register1.as_iced_64()?, xchg.register2.as_iced_64()?)
            .map_err(convert_error)?
    } else if xchg.register1.is_xmm() && xchg.register2.is_xmm() {
        encode_xchg_xmm(a, xchg)?
    } else if xchg.register1.is_ymm() && xchg.register2.is_ymm() {
        encode_xchg_ymm(a, xchg)?
    } else if xchg.register1.is_zmm() && xchg.register2.is_zmm() {
        encode_xchg_zmm(a, xchg)?
    } else {
        return Err(JitError::InvalidRegisterCombination(
            xchg.register1,
            xchg.register2,
        ));
    }

    Ok(())
}

fn get_scratch(scratch: Option<AllRegisters>) -> Result<AllRegisters, JitError<AllRegisters>> {
    match scratch {
        Some(s) => Ok(s),
        None => Err(JitError::NoScratchRegister(
            "Needed for XChgOperation.".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        x64::{self, jit::JitX64},
        x86::{self, jit::JitX86},
    };
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;

    #[rstest]
    #[case(x86::Register::eax, x86::Register::ebx, None, "87d8")]
    #[case(
        x86::Register::xmm0,
        x86::Register::xmm1,
        Some(x86::Register::xmm2),
        "0f28d00f28c10f28ca"
    )]
    #[case(
        x86::Register::ymm0,
        x86::Register::ymm1,
        Some(x86::Register::ymm2),
        "c5fc28d0c5fc28c1c5fc28ca"
    )]
    #[case(
        x86::Register::zmm0,
        x86::Register::zmm1,
        Some(x86::Register::zmm2),
        "62f17c4828d062f17c4828c162f17c4828ca"
    )]
    fn test_compile_xchg_x86(
        #[case] register1: x86::Register,
        #[case] register2: x86::Register,
        #[case] scratch: Option<x86::Register>,
        #[case] expected_encoded: &str,
    ) {
        let xchg = XChg::new(register1, register2, scratch);
        let operations = vec![Op::Xchg(xchg)];
        let result = JitX86::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(x64::Register::rax, x64::Register::rbx, None, "4887d8")]
    #[case(
        x64::Register::xmm0,
        x64::Register::xmm1,
        Some(x64::Register::xmm2),
        "0f28d00f28c10f28ca"
    )]
    #[case(
        x64::Register::ymm0,
        x64::Register::ymm1,
        Some(x64::Register::ymm2),
        "c5fc28d0c5fc28c1c5fc28ca"
    )]
    #[case(
        x64::Register::zmm0,
        x64::Register::zmm1,
        Some(x64::Register::zmm2),
        "62f17c4828d062f17c4828c162f17c4828ca"
    )]
    fn test_compile_xchg_x64(
        #[case] register1: x64::Register,
        #[case] register2: x64::Register,
        #[case] scratch: Option<x64::Register>,
        #[case] expected_encoded: &str,
    ) {
        let xchg = XChg::new(register1, register2, scratch);
        let operations = vec![Op::Xchg(xchg)];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
