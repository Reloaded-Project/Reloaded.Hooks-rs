use core::ptr::read_unaligned;
use reloaded_hooks_portable::api::jit::compiler::DecodeCallTargetResult;

pub fn decode_call_target(
    ins_address: usize,
    ins_length: usize,
) -> Result<DecodeCallTargetResult, &'static str> {
    if ins_length < 5 {
        return Err(
            "[x86 decode_call_target] Instruction length is too short for a relative call.",
        );
    }

    let opcode = unsafe { *(ins_address as *const u8) };
    let is_call = match opcode {
        0xE8 => true,
        0xE9 => false,
        _ => return Err("[x86 decode_call_target] Instruction is not a branch."),
    };

    // Decode the 32-bit offset
    let offset = i32::from_le(unsafe { read_unaligned((ins_address + 1) as *const i32) });

    // Calculate and return the target address
    let target = (ins_address as isize)
        .wrapping_add(5)
        .wrapping_add(offset as isize) as usize;
    Ok(DecodeCallTargetResult::new(target, is_call))
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

        assert_eq!(decoded_target.target_address, expected_address as usize);
    }

    #[test]
    fn test_decode_jmp_target() {
        // Mock the instruction bytes (0xE9 followed by the offset bytes)
        let mut instruction = vec![0xE9];
        instruction.extend_from_slice(&0x1000u32.to_le_bytes());

        // Decode the jmp target
        let decoded_target = decode_call_target(instruction.as_ptr() as usize, 5).unwrap();

        // Mock a jmp instruction with target offset 0x2005)
        let expected_address = instruction.as_ptr().wrapping_add(5).wrapping_add(0x1000);

        assert_eq!(decoded_target.target_address, expected_address as usize);
    }
}
