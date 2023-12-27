extern crate alloc;

use super::{push::encode_push, push_two::encode_push_two};
use crate::all_registers::AllRegisters;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation::MultiPushVec};
use smallvec::SmallVec;

pub fn encode_multi_push(
    x: &SmallVec<MultiPushVec<AllRegisters>>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let mut index = 0;

    while index + 1 < x.len() {
        let reg1 = &x[index].register;
        let reg2 = &x[index + 1].register;

        // Can fail if both registers are a different size.
        // The JIT sorts registers by size, so this should only happen when transitioning,
        // as we can't mix registers either way, this works out for us.
        match encode_push_two(reg1, reg2, pc, buf) {
            Ok(_) => {
                index += 2;
            }
            Err(_) => {
                encode_push(&x[index], pc, buf)?;
                index += 1;
            }
        };
    }

    if index < x.len() {
        let remaining_item = &x[index];
        encode_push(remaining_item, pc, buf)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::jit_instructions::multi_push::encode_multi_push;
    use crate::test_helpers::assert_encode;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;
    use smallvec::smallvec;

    #[rstest]
    // Single Push
    #[case(&[x0], "e08f1ff8")]
    #[case(&[w0], "e0cf1fb8")]
    #[case(&[v0], "e00f9f3c")]
    // Multi Push
    #[case(&[x0, x1], "e007bfa9")]
    #[case(&[w0, w1], "e007bf29")]
    #[case(&[v0, v1], "e007bfad")]
    // Multi Push on High Reg
    #[case(&[x28, x29], "fc77bfa9")]
    #[case(&[w28, w29], "fc77bf29")]
    #[case(&[v28, v29], "fc77bfad")]
    // Multi Push with Leftover
    #[case(&[x0, x1, x2], "e007bfa9e28f1ff8")]
    #[case(&[w0, w1, w2], "e007bf29e2cf1fb8")]
    #[case(&[v0, v1, v2], "e007bfade20f9f3c")]
    fn standard_cases_push(#[case] registers: &[AllRegisters], #[case] expected_hex: &str) {
        test_encode_success_common_push(registers, expected_hex);
    }

    #[rstest]
    // Upcast
    #[case(&[w0, x1], "e0cf1fb8e18f1ff8")]
    #[case(&[x0, v1], "e08f1ff8e10f9f3c")]
    // Downcast
    #[case(&[x1, w0], "e18f1ff8e0cf1fb8")]
    #[case(&[v0, x1], "e00f9f3ce18f1ff8")]
    fn fallback_to_single_push_with_different_sized_registers(
        #[case] registers: &[AllRegisters],
        #[case] expected_hex: &str,
    ) {
        test_encode_success_common_push(registers, expected_hex);
    }

    fn test_encode_success_common_push(registers: &[AllRegisters], expected_hex: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let mut vectors = smallvec![];
        for reg in registers.iter() {
            vectors.push(Push::new(*reg));
        }

        assert!(encode_multi_push(&vectors, &mut pc, &mut buf).is_ok());
        assert_encode(expected_hex, &buf, pc);
    }
}
