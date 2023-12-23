// File: decode_relative_call_target.rs

use core::ptr::read_unaligned;

pub fn decode_call_target(ins_address: usize, ins_length: usize) -> Result<usize, &'static str> {
    if ins_length < 5 {
        return Err(
            "[x86 decode_call_target] Instruction length is too short for a relative call.",
        );
    }

    let opcode = unsafe { *(ins_address as *const u8) };
    if opcode != 0xE8 {
        return Err("[x86 decode_call_target] Instruction is not a relative call.");
    }

    // Decode the 32-bit offset
    let offset = i32::from_le(unsafe { read_unaligned((ins_address + 1) as *const i32) });

    // Calculate and return the target address
    Ok((ins_address as isize)
        .wrapping_add(5)
        .wrapping_add(offset as isize) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_call_target() {
        // Mock the instruction bytes (0xE8 followed by the offset bytes)
        let mut instruction = vec![0xE8];
        instruction.extend_from_slice(&0x1000u32.to_le_bytes());

        // Decode the call target
        let decoded_target = decode_call_target(instruction.as_ptr() as usize, 5).unwrap();

        // Mock a call instruction with target offset 0x2005)
        let expected_address = instruction.as_ptr().wrapping_add(5).wrapping_add(0x1000);

        assert_eq!(decoded_target, expected_address as usize);
    }
}
