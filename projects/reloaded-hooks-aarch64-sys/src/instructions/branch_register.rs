use bitfield::bitfield;

bitfield! {
    pub struct BranchRegister(u32);
    impl Debug;
    u8;

    /// Opcode for the instruction.
    opcode, set_opcode: 31, 25;

    /// Undocumented, but something to do with Shift Type.
    z, set_z: 24, 23;

    /// Sub-operation.
    u16, op, set_op: 22, 12;

    /// Unknown, 'pac'
    a, set_a: 11;

    /// Unknown, 'use_key_a'
    m, set_m: 10;

    /// Address to be branched to.
    rn, set_rn: 9, 5;

    /// Number of bytes to shift by (unsigned).
    rm, set_rm: 4, 0;
}

impl BranchRegister {
    fn initialize(register: u8, op: u16) -> Self {
        let mut value = Self(0);
        value.set_opcode(0b1101011);
        value.set_z(0b00);
        value.set_op(op);
        value.set_a(false);
        value.set_m(false);
        value.set_rm(0);

        value.set_rn(register);
        value
    }

    /// Create a new BLR instruction with the specified register.
    pub fn new_blr(register: u8) -> Self {
        Self::initialize(register, 0b01111110000)
    }

    /// Create a new BR instruction with the specified register.
    pub fn new_br(register: u8) -> Self {
        Self::initialize(register, 0b00111110000)
    }
}
