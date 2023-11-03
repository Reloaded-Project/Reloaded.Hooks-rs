extern crate alloc;

use super::{pop::encode_pop, pop_two::encode_pop_two};
use crate::all_registers::AllRegisters;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation::MultiPopVec};
use smallvec::SmallVec;

pub fn encode_multi_pop(
    x: &SmallVec<MultiPopVec<AllRegisters>>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let mut index = 0;

    while index + 1 < x.len() {
        let reg1 = &x[index].register;
        let reg2 = &x[index + 1].register;

        // Can fail if both registers are a different size.
        match encode_pop_two(reg1, reg2, pc, buf) {
            Ok(_) => {
                index += 2;
            }
            Err(_) => {
                encode_pop(&x[index], pc, buf)?;
                index += 1;
            }
        };
    }

    if index < x.len() {
        let remaining_item = &x[index];
        encode_pop(remaining_item, pc, buf)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::jit_instructions::multi_pop::encode_multi_pop;
    use crate::test_helpers::assert_encode;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;
    use smallvec::smallvec;

    #[rstest]
    // Single Pop
    #[case(&[x0], "e08740f8")]
    #[case(&[w0], "e04740b8")]
    #[case(&[v0], "e007c13c")]
    // Multi Pop
    #[case(&[x0, x1], "e007c1a8")]
    #[case(&[w0, w1], "e007c128")]
    #[case(&[v0, v1], "e007c1ac")]
    // Multi Pop on High Reg
    #[case(&[x28, x29], "fc77c1a8")]
    #[case(&[w28, w29], "fc77c128")]
    #[case(&[v28, v29], "fc77c1ac")]
    // Multi Pop with Leftover
    #[case(&[x0, x1, x2], "e007c1a8e28740f8")]
    #[case(&[w0, w1, w2], "e007c128e24740b8")]
    #[case(&[v0, v1, v2], "e007c1ace207c13c")]
    fn standard_cases(#[case] registers: &[AllRegisters], #[case] expected_hex: &str) {
        test_encode_success_common(registers, expected_hex);
    }

    #[rstest]
    // Upcast
    #[case(&[w0, x1], "e04740b8e18740f8")]
    #[case(&[x0, v1], "e08740f8e107c13c")]
    // Downcast
    #[case(&[x1, w0], "e18740f8e04740b8")]
    #[case(&[v0, x1], "e007c13ce18740f8")]
    fn fallback_to_single_pop_with_different_sized_registers(
        #[case] registers: &[AllRegisters],
        #[case] expected_hex: &str,
    ) {
        test_encode_success_common(registers, expected_hex);
    }

    fn test_encode_success_common(registers: &[AllRegisters], expected_hex: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let mut vectors = smallvec![];
        for reg in registers.iter() {
            vectors.push(Pop::new(*reg));
        }

        assert!(encode_multi_pop(&vectors, &mut pc, &mut buf).is_ok());
        assert_encode(expected_hex, &buf, pc);
    }
}
