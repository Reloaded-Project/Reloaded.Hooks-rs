extern crate alloc;

use crate::common::jit_common::X86jitError;
use crate::{x64::Register as x64Register, x86::Register as x86Register};
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::operation_aliases::PushConst;
use zydis::{EncoderRequest, Mnemonic::PUSH};

#[cfg(feature = "x86")]
pub(crate) fn encode_push_constant_x86(
    push_const: &PushConst<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    const INSTRUCTION_LENGTH: usize = 5; // Opcode (1 byte) + Immediate value (4 bytes)

    unsafe {
        let old_len = buf.len();
        buf.reserve(INSTRUCTION_LENGTH); // Reserve space for 5 bytes
        let ptr = buf.as_mut_ptr().add(old_len);

        // Write opcode for PUSH with immediate value
        ptr.write(0x68);

        // Write the 4-byte immediate value
        let value_ptr = ptr.add(1) as *mut u32;
        value_ptr.write_unaligned((push_const.value as u32).to_le());

        buf.set_len(old_len + INSTRUCTION_LENGTH);
    }

    *pc += INSTRUCTION_LENGTH; // Update program counter
    Ok(())
}
// x64 implementation
#[cfg(feature = "x64")]
pub(crate) fn encode_push_constant_x64(
    push_const: &PushConst<x64Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64Register>> {
    *pc += EncoderRequest::new64(PUSH)
        .add_operand((push_const.value as i64 >> 32) & 0xFFFFFFFF)
        .encode_extend(buf)?;

    *pc += EncoderRequest::new64(PUSH)
        .add_operand(push_const.value as i32)
        .encode_extend(buf)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode;
    use crate::x64;
    use crate::x86;

    #[test]
    fn push_constant_x86_test() {
        let mut pc = 0;
        let mut buf = Vec::new();
        let push_const = PushConst::<x86::Register> {
            value: 0x87654321,
            scratch: None,
        };
        encode_push_constant_x86(&push_const, &mut pc, &mut buf).unwrap();
        assert_encode("6821436587", &buf, pc);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn push_constant_x64_test() {
        let mut pc = 0;
        let mut buf = Vec::new();
        let push_const = PushConst::<x64::Register> {
            value: 0x11111111EFEFEFEF,
            scratch: None,
        };
        encode_push_constant_x64(&push_const, &mut pc, &mut buf).unwrap();
        assert_encode("681111111168efefefef", &buf, pc);
    }
}
