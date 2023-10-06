use reloaded_hooks_portable::api::jit::{
    compiler::JitError, mov_operation::MovOperation, xchg_operation::XChgOperation,
};
extern crate alloc;
use crate::{
    all_registers::AllRegisters,
    instructions::{orr::Orr, orr_vector::OrrVector},
};
use alloc::string::ToString;
use alloc::vec::Vec;

pub fn encode_xchg(
    x: &XChgOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let scratch = match x.scratch {
        Some(s) => s,
        None => {
            return Err(JitError::NoScratchRegister(
                "Needed for XChgOperation.".to_string(),
            ));
        }
    };

    let source_size = x.register1.size();
    let target_size = x.register2.size();
    let scratch_size = scratch.size();

    let r1 = x.register1.register_number();
    let r2 = x.register2.register_number();
    let rs = scratch.register_number();

    if source_size == 8 && target_size == 8 && scratch_size == 8 {
        let temp = Orr::new_mov(true, rs as u8, r1 as u8).0;
        let r1mov = Orr::new_mov(true, r1 as u8, r2 as u8).0;
        let r2mov = Orr::new_mov(true, r2 as u8, rs as u8).0;
        buf.push(temp.to_le() as i32);
        buf.push(r1mov.to_le() as i32);
        buf.push(r2mov.to_le() as i32);
        *pc += 12;
    } else if source_size == 16 && target_size == 16 && scratch_size == 16 {
        let temp = OrrVector::new_mov(rs as u8, r1 as u8).0;
        let r1mov = OrrVector::new_mov(r1 as u8, r2 as u8).0;
        let r2mov = OrrVector::new_mov(r2 as u8, rs as u8).0;
        buf.push(temp.to_le() as i32);
        buf.push(r1mov.to_le() as i32);
        buf.push(r2mov.to_le() as i32);
        *pc += 12;
    } else if source_size == 4 && target_size == 4 && scratch_size == 4 {
        let temp = Orr::new_mov(false, rs as u8, r1 as u8).0;
        let r1mov = Orr::new_mov(false, r1 as u8, r2 as u8).0;
        let r2mov = Orr::new_mov(false, r2 as u8, rs as u8).0;
        buf.push(temp.to_le() as i32);
        buf.push(r1mov.to_le() as i32);
        buf.push(r2mov.to_le() as i32);
        *pc += 12;
    } else {
        return Err(JitError::InvalidRegisterCombination3(
            x.register1,
            x.register2,
            scratch,
        ));
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::jit_instructions::xchg::encode_xchg;
    use crate::test_helpers::instruction_buffer_as_hex;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(w0, w1, w2, "e203012ae103002ae003022a", false)]
    #[case(x0, x1, x2, "e20301aae10300aae00302aa", false)]
    #[case(v0, v1, v2, "221ca14e011ca04e401ca24e", false)]
    // Some fail cases
    fn test_encode_xchg(
        #[case] reg_1: AllRegisters,
        #[case] reg_2: AllRegisters,
        #[case] scratch: AllRegisters,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = XChg::new(reg_2, reg_1, Some(scratch));

        // If source and target size don't match, expect an error
        if is_err {
            assert!(encode_xchg(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        assert!(encode_xchg(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(12, pc);
    }
}
