use crate::common::util::get_stolen_instructions::get_stolen_instructions_length;
use core::slice;
use reloaded_hooks_portable::api::length_disassembler::LengthDisassembler;

// Assume Instruction and CodeRewriterError are defined elsewhere
pub struct LengthDisassemblerX64;

impl LengthDisassembler for LengthDisassemblerX64 {
    fn disassemble_length(code_address: usize, min_length: usize) -> usize {
        // + 16 for max instruction size.
        let code = unsafe { slice::from_raw_parts(code_address as *const u8, min_length + 16) };

        // Only possible error to return is 'insufficient bytes', however, we add max instruction size
        // (16 bytes) to counteract this, so unwrap is ok.
        get_stolen_instructions_length(true, min_length as u8, code, code_address).unwrap() as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::common::util::test_utilities::str_to_vec;
    use crate::x64::length_disassembler::LengthDisassemblerX64;
    use reloaded_hooks_portable::api::length_disassembler::LengthDisassembler;
    use rstest::rstest;

    #[rstest]
    #[case::simple_jump("eb02", 2, 2)]
    #[case::two_instructions("4883ec108b442404", 5, 8)]
    #[case::on_instruction_boundary("4883ec108b4424044889d8", 8, 8)]
    fn can_disassemble_length(
        #[case] instructions: String,
        #[case] min_length: usize,
        #[case] expected_length: usize,
    ) {
        let ins_vec = str_to_vec(instructions);
        let code_address = ins_vec.as_ptr() as usize;
        let result = LengthDisassemblerX64::disassemble_length(code_address, min_length);
        assert_eq!(result, expected_length);
    }
}
