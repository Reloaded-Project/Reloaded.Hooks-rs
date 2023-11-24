/// Trait for length disassembly.
///
/// Length disassembly involves analyzing a sequence of machine instructions at a given address
/// and calculating the total length of these instructions, in bytes, up to a specified minimum length.
/// This is crucial in function hooking, where one needs to determine the exact number of bytes to
/// replace with a jump instruction without disrupting the original instruction flow.
///
/// # Example
/// Consider hooking the function `DoMathWithTwoNumbers`. To insert a 5-byte relative jump at the start,
/// we need to ensure it does not break any existing instructions. A length disassembler can be used to
/// calculate the total length of instructions from the function start to a minimum length, ensuring safe
/// overwrite with the jump.
///
/// ```
/// // Original x86 Assembly Code
/// // DoMathWithTwoNumbers:
/// //   cmp rcx, 0 ; 48 83 F9 00
/// //   jg skipAdd ; 7F 0E
/// //
/// // When a 5-byte jump is inserted:
/// // DoMathWithTwoNumbers:
/// //   jmp stub ; E9 XX XX XX XX
/// //   <INVALID INSTRUCTION> ; 0E
/// ```
///
/// The length disassembler ensures that all bytes of the original two instructions are accounted for,
/// avoiding partial instruction overwrite.
pub trait LengthDisassembler {
    /// Disassembles instructions starting from `code_address` and calculates their total length.
    /// The disassembly continues until the combined length of these instructions is at least `min_length` bytes.
    ///
    /// # Parameters
    /// - `code_address`: The starting address in memory where the machine code is located.
    /// - `min_length`: The minimum total length in bytes of the instructions to be disassembled.
    ///
    /// # Returns
    /// The method returns the total length (in bytes) of the disassembled instructions.
    /// This length is guaranteed to be greater than or equal to `min_length`.
    ///
    /// # Example
    ///
    /// ```compile_fail
    /// use reloaded_hooks_portable::api::length_disassembler::LengthDisassembler;
    /// let function_start_address = 0x40020000;
    /// let length_needed = 5; // Typical length for a x86 jump instruction
    /// let length = LengthDisassembler::disassemble_length(function_start_address, length_needed);
    /// // Use `length` to safely overwrite original code for function hooking
    /// ```
    ///
    /// # Safety
    ///
    /// [`code_address`] better point to a valid address, or you're screwed.
    ///
    /// # Returns
    ///
    /// Tuple, with first element being length in bytes, and second being number of instructions.
    fn disassemble_length(code_address: usize, min_length: usize) -> (usize, usize);
}
