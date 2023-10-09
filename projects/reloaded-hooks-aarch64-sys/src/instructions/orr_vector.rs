use bitfield::bitfield;

// https://developer.arm.com/documentation/ddi0602/2022-03/SIMD-FP-Instructions/MOV--vector---Move-vector--an-alias-of-ORR--vector--register--?lang=en
bitfield! {
    pub struct OrrVector(u32);
    impl Debug;
    u8;

    /// Always zero
    zero, set_zero: 31;

    /// Controls the size of the vectors register. 0 = 8 bytes, 1 = 16 bytes.
    q, set_q: 30;

    /// The operation opcode for the ORR Vector instruction.
    /// This is constant 0b001110 for this operation.
    opcode, set_opcode: 29, 24;

    /// Size: This is constant 0b101 for this operation.
    size, set_size: 23, 21;

    /// RM: The second register.
    r, set_rm: 20, 16;

    /// Constant of 0b000111
    unk, set_unk: 15, 10;

    /// RN: First source register.
    rn, set_rn: 9, 5;

    /// RD: Destination register
    rd, set_rd: 4, 0;
}

impl OrrVector {
    /// Create a new MOV instruction with the specified parameters.
    /// Note that MOV is an alias for ORR with the second operand being 'ZR' or 'WZR'.
    pub fn new_mov(destination: u8, source: u8) -> Self {
        let mut value = OrrVector(0);

        // Compile time eval
        value.set_unk(0b000111);
        value.set_size(0b101);
        value.set_opcode(0b001110);
        value.set_zero(false);

        // Full vector size
        value.set_q(true);

        // Set params
        value.set_rm(source); // Docs specify to repeat rn twice, `ORR <Vd>.<T>, <Vn>.<T>, <Vn>.<T>`
        value.set_rn(source);
        value.set_rd(destination);

        value
    }
}
