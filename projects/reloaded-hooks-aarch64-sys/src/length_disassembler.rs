use reloaded_hooks_portable::api::length_disassembler::LengthDisassembler;

// Assume Instruction and CodeRewriterError are defined elsewhere
pub struct LengthDisassemblerAarch64;

impl LengthDisassembler for LengthDisassemblerAarch64 {
    fn disassemble_length(_code_address: usize, min_length: usize) -> (usize, usize) {
        // In AArch64, instructions are 4 bytes long, so we can just round up to the nearest multiple of 4.
        // If the minimum length is already a multiple of 4, we can just return it.
        let length = (min_length + 3) & !3;
        (length, length / 4)
    }
}

#[cfg(test)]
mod tests {
    use crate::length_disassembler::LengthDisassemblerAarch64;
    use reloaded_hooks_portable::api::length_disassembler::LengthDisassembler;
    use rstest::rstest;

    #[rstest]
    #[case::single_ins("410040f9", 4, 4, 1)]
    #[case::two_instructions("830040f9830040f9", 8, 8, 2)]
    #[case::non_aligned_min_length("830040f9830040f9", 5, 8, 2)]
    fn can_disassemble_length(
        #[case] instructions: String,
        #[case] min_length: usize,
        #[case] expected_length: usize,
        #[case] expected_num_ins: usize,
    ) {
        let ins_vec = str_to_vec(instructions);
        let code_address = ins_vec.as_ptr() as usize;
        let result = LengthDisassemblerAarch64::disassemble_length(code_address, min_length);
        assert_eq!(result.0, expected_length);
        assert_eq!(result.1, expected_num_ins);
    }

    fn str_to_vec(hex: String) -> Vec<u8> {
        hex.as_bytes()
            .chunks(2)
            .map(|chunk| {
                let hex_str = std::str::from_utf8(chunk).unwrap();
                u8::from_str_radix(hex_str, 16).unwrap()
            })
            .collect()
    }
}
