use reloaded_hooks_portable::api::jit::compiler::JitError;
extern crate alloc;
use crate::all_registers::AllRegisters;
use crate::instructions::adr::Adr;
use crate::instructions::ldr_immediate_unsigned_offset::LdrImmediateUnsignedOffset;
use alloc::vec::Vec;

/// Loads a value at a PC relative address into a register.
///
/// # Arguments
///
/// * `x` - A reference to a struct containing all the CPU registers.
/// * `pc` - A mutable reference to the program counter.
/// * `buf` - A mutable reference to a vector of 32-bit integers that will hold the assembled instructions.
/// * `target_address` - The address that we want to load into the register.
///
/// # Returns
///
/// Returns a `Result` with an empty `Ok` value if the assembly is successful, or a `JitError` if
/// there is an error assembling the instructions.
pub fn load_pc_rel_value(
    x: AllRegisters,
    pc: &mut usize,
    buf: &mut Vec<i32>,
    target_address: usize,
) -> Result<(), JitError<AllRegisters>> {
    let offset = target_address.wrapping_sub(*pc) as isize;

    // Assemble ADR+LDR if within 1MiB range.
    if (-1048576..=1048575).contains(&offset) {
        let adr = Adr::new_adr(x.register_number() as u8, offset as i32)?.0;
        buf.push(adr.to_le() as i32);
        let ldr = LdrImmediateUnsignedOffset::new_mov_from_reg(
            true,
            x.register_number() as u8,
            0,
            x.register_number() as u8,
        )?
        .0;
        buf.push(ldr.to_le() as i32);
        *pc += 8;
        return Ok(());
    }

    // Assemble ADRP + LDR if out of range.
    // This will error if our address is too far.
    let reg_num = x.register_number() as u8;
    let rounded_down = offset & !4095;

    let adrp = Adr::new_adrp(reg_num, rounded_down as i64)?.0;
    buf.push(adrp.to_le() as i32);

    let remainder = offset - rounded_down;
    let ldr = LdrImmediateUnsignedOffset::new_mov_from_reg(
        true,
        x.register_number() as u8,
        remainder as i32,
        x.register_number() as u8,
    )?
    .0;
    buf.push(ldr.to_le() as i32);
    *pc += 8;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters::*;
    use crate::assert_error;
    use crate::jit_instructions::load_pc_relative_value::load_pc_rel_value;
    use crate::test_helpers::assert_encode_with_initial_pc;
    use reloaded_hooks_portable::api::jit::compiler::JitError;
    use rstest::rstest;

    #[rstest]
    #[case(0, 4, "20000010000040f9")] // next instruction
    #[case(4, 0, "e0ffff10000040f9")] // last instruction
    #[case(0, 1048575, "e0ff7f70000040f9")] // max value
    #[case(1048576, 0, "00008010000040f9")] // min value
    fn can_encode_load_pc_rel_value_with_adr(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        assert!(load_pc_rel_value(x0, &mut pc, &mut buf, target_address).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    #[rstest]
    #[case(0, 2097152, "00100090000040f9")] // 2MB after
    #[case(2097152, 0, "00f0ff90000040f9")] // 2MB before
    fn can_encode_pc_rel_with_adrp(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        assert!(load_pc_rel_value(x0, &mut pc, &mut buf, target_address).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    #[rstest]
    #[case(0, 2097152 + 2048, "00100090000044f9")] // 2MB after
    #[case(2097152 + 2048, 0, "e0effff0000044f9")] // 2MB before
    fn can_encode_pc_rel_with_adrp_and_adr(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        assert!(load_pc_rel_value(x0, &mut pc, &mut buf, target_address).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    #[rstest]
    #[case(0, 4294967296)] // 1 beyond max forward jump
    #[case(4294967297, 0)] // 1 beyond max back jump
    fn error_when_out_of_range(#[case] initial_pc: usize, #[case] target_address: usize) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        let result = load_pc_rel_value(x0, &mut pc, &mut buf, target_address);
        assert_error!(
            result,
            JitError::OperandOutOfRange(_),
            initial_pc,
            0,
            pc,
            &buf
        );
    }
}
