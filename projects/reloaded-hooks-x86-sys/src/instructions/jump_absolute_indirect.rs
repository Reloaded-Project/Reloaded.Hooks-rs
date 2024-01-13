extern crate alloc;

use crate::common::jit_common::X86jitError;
use crate::x64;
use crate::x86;
use alloc::vec::Vec;
use core::ptr::write_unaligned;
use reloaded_hooks_portable::api::jit::operation_aliases::JumpAbsInd;

// For x86 architecture
#[cfg(feature = "x86")]
pub(crate) fn encode_jump_absolute_indirect_x86(
    x: &JumpAbsInd<x86::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86::Register>> {
    // Opcode for JMP [imm32] (FF /4)
    const OPCODE_LEN: usize = 6;

    buf.reserve(OPCODE_LEN); // Reserve space for the instruction
    unsafe {
        let ptr = buf.as_mut_ptr().add(buf.len());

        // Write the opcode FF and ModRM byte 25
        write_unaligned(ptr as *mut u16, 0x25FF_u16.to_le());

        // Write the 32-bit address
        let address = x.pointer_address as u32;
        write_unaligned(ptr.add(2) as *mut u32, address.to_le());

        buf.set_len(buf.len() + OPCODE_LEN);
    }

    *pc += OPCODE_LEN;
    Ok(())
}

// For x64 architecture
#[cfg(feature = "x64")]
pub(crate) fn encode_jump_absolute_indirect_x64(
    x: &JumpAbsInd<x64::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64::Register>> {
    // Opcode for JMP [imm64] (REX.W + FF /4)
    const OPCODE_LEN: usize = 7;

    buf.reserve(OPCODE_LEN); // Reserve space for the instruction
    unsafe {
        let ptr = buf.as_mut_ptr().add(buf.len());

        // Write the opcode FF and ModRM byte 25
        write_unaligned(ptr as *mut u32, 0x002524FF_u32.to_le());

        // Write the 32-bit address
        let address = x.pointer_address as u32;
        write_unaligned(ptr.add(3) as *mut u32, address.to_le());

        buf.set_len(buf.len() + OPCODE_LEN);
    }

    *pc += OPCODE_LEN;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode;
    use rstest::rstest;

    #[cfg(feature = "x86")]
    #[rstest]
    #[case(0x12345678, "ff2578563412")]
    fn test_encode_jump_absolute_indirect_x86(
        #[case] pointer_address: usize,
        #[case] expected_encoded: &str,
    ) {
        let mut buf = Vec::new();
        let mut pc = 0;
        let jump = JumpAbsInd::<x86::Register> {
            pointer_address,
            scratch_register: None,
        };

        encode_jump_absolute_indirect_x86(&jump, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }

    #[cfg(feature = "x64")]
    #[rstest]
    #[case(0x12345678, "ff242578563412")]
    fn test_encode_jump_absolute_indirect_x64(
        #[case] pointer_address: usize,
        #[case] expected_encoded: &str,
    ) {
        let mut buf = Vec::new();
        let mut pc = 0;
        let jump = JumpAbsInd::<x64::Register> {
            pointer_address,
            scratch_register: None,
        };

        encode_jump_absolute_indirect_x64(&jump, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }
}
