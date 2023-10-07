use reloaded_hooks_portable::api::jit::{compiler::JitError, operation::MultiPushVec};
use smallvec::SmallVec;
extern crate alloc;
use crate::all_registers::AllRegisters;
use alloc::vec::Vec;

use super::{push::encode_push, push_two::encode_push_two};

pub fn encode_multi_push(
    x: &SmallVec<MultiPushVec<AllRegisters>>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let mut index = 0;

    while index + 1 < x.len() {
        let reg1 = &x[index].register;
        let reg2 = &x[index + 1].register;
        encode_push_two(reg1, reg2, pc, buf)?;
        index += 2;
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
    use crate::test_helpers::instruction_buffer_as_hex;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;
    use smallvec::smallvec;

    #[rstest]
    // Single Push
    #[case(&[x0], "e08f1ff8", false)]
    #[case(&[w0], "e0cf1fb8", false)]
    #[case(&[v0], "e00f9f3c", false)]
    // Multi Push
    #[case(&[x0, x1], "e007bfa9", false)]
    #[case(&[w0, w1], "e007bf29", false)]
    #[case(&[v0, v1], "e007bfad", false)]
    // Multi Push with Leftover
    #[case(&[x0, x1, x2], "e007bfa9e28f1ff8", false)]
    #[case(&[w0, w1, w2], "e007bf29e2cf1fb8", false)]
    #[case(&[v0, v1, v2], "e007bfade20f9f3c", false)]
    // TODO: Mixed Size Registers
    fn test_encode_multi_push(
        #[case] registers: &[AllRegisters],
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let mut vectors = smallvec![];
        for reg in registers.iter() {
            vectors.push(Push::new(*reg));
        }

        // Expect an error for invalid register sizes
        if is_err {
            assert!(encode_multi_push(&vectors, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_multi_push(&vectors, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
    }
}
