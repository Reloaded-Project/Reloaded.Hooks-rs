extern crate alloc;

use crate::{common::jit_common::X86jitError, x64::Register as x64Register};
use alloc::vec::Vec;
use core::ptr::write_unaligned;
use reloaded_hooks_portable::api::jit::operation_aliases::JumpIpRel;

#[cfg(feature = "x64")]
pub(crate) fn encode_jump_ip_relative_x64(
    x: &JumpIpRel<x64Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64Register>> {
    unsafe {
        buf.reserve(6); // Reserve space for 6-bytes
        let ptr = buf.as_mut_ptr().add(buf.len());

        // Opcode & ModR/M byte
        write_unaligned(ptr as *mut u16, 0x25FF_u16.to_le());

        // Calculate relative offset (32-bit, signed)
        let relative_offset = (x.target_address as i32)
            .wrapping_sub((*pc + 6) as i32)
            .to_le();

        // Write the 32-bit relative offset as little-endian
        write_unaligned(ptr.add(2) as *mut i32, relative_offset);

        buf.set_len(buf.len() + 6); // Update buffer length
        *pc += 6; // Update program counter by the length of the instruction
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode;
    use crate::common::util::test_utilities::assert_encode_with_initial_pc;
    use reloaded_hooks_portable::api::jit::operation_aliases::JumpIpRel;
    use rstest::rstest;

    #[rstest]
    #[case(16, "ff250a000000")]
    fn test_encode_jump_ip_relative_forward(
        #[case] target_address: usize,
        #[case] expected_encoded: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let jump = JumpIpRel::new(target_address);
        encode_jump_ip_relative_x64(&jump, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }

    #[rstest]
    #[case(0, "ff25f0ffffff")]
    fn test_encode_jump_ip_relative_backward(
        #[case] target_address: usize,
        #[case] expected_encoded: &str,
    ) {
        let mut pc = 10;
        let mut buf = Vec::new();
        let jump = JumpIpRel::new(target_address);
        encode_jump_ip_relative_x64(&jump, &mut pc, &mut buf).unwrap();
        assert_encode_with_initial_pc(expected_encoded, &buf, 10, pc);
    }
}
