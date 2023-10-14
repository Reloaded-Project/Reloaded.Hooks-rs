use reloaded_hooks_portable::api::jit::compiler::JitError;
extern crate alloc;
use crate::all_registers::AllRegisters;
use crate::instructions::add_immediate::AddImmediate;
use crate::instructions::adr::Adr;
use alloc::vec::Vec;

/// Loads a PC relative address into a register.
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
pub fn load_pc_rel_address(
    x: &AllRegisters,
    pc: &mut usize,
    buf: &mut Vec<i32>,
    target_address: usize,
) -> Result<(), JitError<AllRegisters>> {
    let offset = target_address.wrapping_sub(*pc) as isize;

    // Assemble ADR only if within range.
    if (-1048576..=1048575).contains(&offset) {
        let adr = Adr::new_adr(x.register_number() as u8, offset as i32)?.0;
        buf.push(adr.to_le() as i32);
        *pc += 4;
        return Ok(());
    }

    // Assemble ADRP + ADD if out of range.
    // This will error if our address is too far.
    let reg_num = x.register_number() as u8;
    let adrp_pc = *pc & !4095; // round down to page
    let adrp_target_address = target_address & !4095; // round down to page
    let adrp_offset = adrp_target_address.wrapping_sub(adrp_pc) as isize;

    let adrp = Adr::new_adrp(reg_num, adrp_offset as i64)?.0;
    buf.push(adrp.to_le() as i32);
    *pc += 4;

    let remainder = target_address - adrp_target_address;
    if remainder > 0 {
        let add = AddImmediate::new(true, reg_num, reg_num, remainder as u16)?.0;
        buf.push(add.to_le() as i32);
        *pc += 4;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters::*;
    use crate::assert_error;
    use crate::jit_instructions::load_pc_relative_address::load_pc_rel_address;
    use crate::test_helpers::assert_encode_with_initial_pc;
    use reloaded_hooks_portable::api::jit::compiler::JitError;
    use rstest::rstest;

    #[rstest]
    #[case(0, 4, "20000010")] // next instruction
    #[case(4, 0, "e0ffff10")] // last instruction
    #[case(0, 0xFFFFF, "e0ff7f70")] // max value
    #[case(0x100000, 0, "00008010")] // min value
    fn can_encode_pc_rel_with_adr(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        assert!(load_pc_rel_address(&x0, &mut pc, &mut buf, target_address).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    #[rstest]
    #[case(0, 0x200000, "00100090")] // 2MB after
    #[case(0x200000, 0, "00f0ff90")] // 2MB before
    fn can_encode_pc_rel_with_adrp(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        assert!(load_pc_rel_address(&x0, &mut pc, &mut buf, target_address).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    #[rstest]
    #[case(0, 0x200000 + 2048, "0010009000002091")] // 2MB + 2048 after
    #[case(0x200000 + 2048, 0, "00f0ff90")] // 2MB + 2048 before
    fn can_encode_pc_rel_with_adrp_and_adr(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        assert!(load_pc_rel_address(&x0, &mut pc, &mut buf, target_address).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    /// The ADRP isntruction is relative to the start of the page, not the start of the instruction,
    /// so if PC is at 2048, we should treat it as if it was at 0 (round down 4096).
    #[rstest]
    #[case(2048, 0x200000 + 2048, "0010009000002091")] // 2MB after
    #[case(0x200000 + 2048, 2048, "00f0ff9000002091")] // 2MB before
    fn adrp_is_relative_to_start_of_page(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        assert!(load_pc_rel_address(&x0, &mut pc, &mut buf, target_address).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    #[rstest]
    #[case(0, 4294967296)] // 1 beyond max forward jump
    #[case(4294967296 + 4096, 0)] // 1 beyond max back jump (+4096 due to page round down)
    fn error_when_out_of_range(#[case] initial_pc: usize, #[case] target_address: usize) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        let result = load_pc_rel_address(&x0, &mut pc, &mut buf, target_address);
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
