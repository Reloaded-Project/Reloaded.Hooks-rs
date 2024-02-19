use crate::common::util::zydis_decoder_result::ZydisDecoderResult;
use zydis::{ffi::DecodedOperandKind::*, VisibleOperands};

/// Returns true if the instruction has a single immediate operand.
/// These represent branches we will need to rewrite in code rewriter.
pub(crate) fn has_single_immediate_operand(ins: &ZydisDecoderResult<VisibleOperands>) -> bool {
    if ins.instruction.operand_count_visible == 1 {
        if let Imm(_) = ins.instruction.operands()[0].kind {
            return true;
        }
    }
    false
}
