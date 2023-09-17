use bitfield::bitfield;

bitfield! {
    /// `Orr` represents the bitfields of the ORR (shifted register) instruction
    /// in AArch64 architecture. The bitfields are described as follows:
    pub struct Orr(u32);
    impl Debug;
    u8;

    /// Set flag determines whether the operation is 32 or 64 bits.
    /// 0 for 32-bit and 1 for 64-bit.
    sf, set_sf: 31;

    /// Opcode for the ORR instruction, generally `0b01010100`.
    opcode, set_opcode: 30, 24;

    /// Defines the type of shift to be applied. Generally `0b00`.
    shift_type, set_shift_type: 23, 22;

    /// Inverts the shift direction. Generally `0b00`.
    invert, set_invert: 21;

    /// Register number for the second operand (source).
    rm, set_rm: 20, 16;

    /// Number of bytes to shift by (unsigned).
    shift_amount, set_shift_amount: 15, 10;

    /// Register number for the first operand (source).
    rn, set_rn: 9, 5;

    /// Register number for the destination where the result will be stored.
    rd, set_rd: 4, 0;
}

impl Orr {
    /// Create a new MOV instruction with the specified parameters.
    /// Note that MOV is an alias for ORR with the first operand being 'ZR' or 'WZR'.
    pub fn new_mov(is_64bit: bool, destination: u8, source: u8) -> Self {
        // Note: Compiler is smart enough to optimize this away as a constant
        // Which is why we moved the non-constant stuff to the bottom.
        let mut value = Orr(0);
        value.set_opcode(0b0101010);
        value.set_shift_type(0b00);
        value.set_shift_amount(0);
        value.set_rn(31); // ZR for 64-bit, WZR for 32-bit

        value.set_sf(is_64bit);
        value.set_rm(source);
        value.set_rd(destination);
        value
    }
}
