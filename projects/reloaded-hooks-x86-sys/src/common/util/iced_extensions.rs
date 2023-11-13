use iced_x86::{Instruction, OpKind};

pub(crate) fn is_immediate(kind: OpKind) -> bool {
    kind == OpKind::Immediate8
        || kind == OpKind::Immediate16
        || kind == OpKind::Immediate32
        || kind == OpKind::Immediate32to64
        || kind == OpKind::Immediate64
        || kind == OpKind::Immediate8to16
        || kind == OpKind::Immediate8to32
        || kind == OpKind::Immediate8to64
        || kind == OpKind::Immediate8_2nd
}
