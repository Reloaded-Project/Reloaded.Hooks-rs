use zydis::Instruction;

pub struct ZydisDecoderResult<'buffer, O: zydis::Operands> {
    pub ip: u64,
    pub instruction_bytes: &'buffer [u8],
    pub instruction: Instruction<O>,
}

impl<'buffer, O: zydis::Operands> From<(u64, &'buffer [u8], Instruction<O>)>
    for ZydisDecoderResult<'buffer, O>
{
    fn from(tuple: (u64, &'buffer [u8], Instruction<O>)) -> Self {
        ZydisDecoderResult {
            ip: tuple.0,
            instruction_bytes: tuple.1,
            instruction: tuple.2,
        }
    }
}
