use reloaded_hooks_portable::api::jit::operation_aliases::MovFromStack;
use reloaded_hooks_portable::api::jit::push_operation::PushOperation;
use reloaded_hooks_portable::api::jit::{
    compiler::JitError, push_stack_operation::PushStackOperation,
};
extern crate alloc;
use crate::all_registers::AllRegisters;
use alloc::string::ToString;
use alloc::vec::Vec;

use super::mov_from_stack::encode_mov_from_stack;
use super::mov_two_from_stack::encode_mov_two_from_stack;
use super::push::encode_push;
use super::push_two::encode_push_two;

// TODO: Disabled because optimised version relies on MultiPop / MultiPush.
pub fn encode_push_stack(
    x: &PushStackOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    // Validate remaining size is usable.
    if x.item_size % 4 != 0 {
        return Err(JitError::ThirdPartyAssemblerError(
            "ARM64 PushStack Must use Multiple of 4 Sizes".to_string(),
        ));
    }

    // A free vector register, let's go !
    let mut remaining_bytes = x.item_size;
    if let Some(regs) = get_two_vector_one_scalar_register(&x.scratch) {
        // Vectorised variant (2 vector regs + 1 scalar reg)
        // Two register variant
        while remaining_bytes > 0 {
            if remaining_bytes >= 32 {
                // Push Two Vectors
                encode_mov_two_from_stack(&regs.0, &regs.1, x.offset, pc, buf)?;
                encode_push_two(&regs.0, &regs.1, pc, buf)?;
                remaining_bytes -= 32;
            } else if remaining_bytes >= 16 {
                // Push Single Vector
                encode_mov_from_stack(&MovFromStack::new(x.offset, regs.0), pc, buf)?;
                encode_push(&PushOperation::new(regs.0), pc, buf)?;
                remaining_bytes -= 16;
            } else if remaining_bytes >= 8 {
                // Push Single Register
                encode_mov_from_stack(&MovFromStack::new(x.offset, regs.2), pc, buf)?;
                encode_push(&PushOperation::new(regs.2), pc, buf)?;
                remaining_bytes -= 8;
            } else {
                // Push Remaining Multiple of 4
                encode_mov_from_stack(&MovFromStack::new(x.offset, regs.2.shrink_to32()), pc, buf)?;
                encode_push(&PushOperation::new(regs.2.shrink_to32()), pc, buf)?;
                remaining_bytes -= 4;
            }
        }
    } else if let Some(two_reg) = get_two_64_bit_registers(&x.scratch) {
        // Two register variant
        while remaining_bytes > 0 {
            if remaining_bytes >= 16 {
                // Push Two Registers
                encode_mov_two_from_stack(&two_reg.0, &two_reg.1, x.offset, pc, buf)?;
                encode_push_two(&two_reg.0, &two_reg.1, pc, buf)?;
                remaining_bytes -= 16;
            } else if remaining_bytes >= 8 {
                // Push Single Register
                encode_mov_from_stack(&MovFromStack::new(x.offset, two_reg.0), pc, buf)?;
                encode_push(&PushOperation::new(two_reg.0), pc, buf)?;
                remaining_bytes -= 8;
            } else {
                // Push Remaining Multiple of 4
                encode_mov_from_stack(
                    &MovFromStack::new(x.offset, two_reg.0.shrink_to32()),
                    pc,
                    buf,
                )?;
                encode_push(&PushOperation::new(two_reg.0.shrink_to32()), pc, buf)?;
                remaining_bytes -= 4;
            }
        }
    } else if let Some(reg) = x.scratch.iter().find(|reg| reg.is_64()) {
        // Single register fallback
        while remaining_bytes > 0 {
            if remaining_bytes >= 8 {
                // Push Single Register
                encode_mov_from_stack(&MovFromStack::new(x.offset, *reg), pc, buf)?;
                encode_push(&PushOperation::new(*reg), pc, buf)?;
                remaining_bytes -= 8;
            } else {
                // Push Remaining Multiple of 4
                encode_mov_from_stack(&MovFromStack::new(x.offset, reg.shrink_to32()), pc, buf)?;
                encode_push(&PushOperation::new(reg.shrink_to32()), pc, buf)?;
                remaining_bytes -= 4;
            }
        }
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            "No scratch register available".to_string(),
        ));
    }

    Ok(())
}

fn get_two_64_bit_registers(vec: &[AllRegisters]) -> Option<(AllRegisters, AllRegisters)> {
    let mut iter = vec.iter().filter(|reg| reg.is_64());
    let reg1 = iter.next()?;
    let reg2 = iter.next()?;
    Some((*reg1, *reg2))
}

fn get_two_vector_one_scalar_register(
    vec: &[AllRegisters],
) -> Option<(AllRegisters, AllRegisters, AllRegisters)> {
    let mut vec_iter = vec.iter().filter(|reg| reg.is_128());
    let mut scalar_iter = vec.iter().filter(|reg| reg.is_64());

    let reg1 = vec_iter.next()?;
    let reg2 = vec_iter.next()?;
    let reg3 = scalar_iter.next()?;

    Some((*reg1, *reg2, *reg3))
}

#[cfg(test)]
mod tests {

    extern crate alloc;
    use alloc::rc::Rc;
    use alloc::vec::Vec;

    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::jit_instructions::push_stack::encode_push_stack;
    use crate::test_helpers::assert_encode;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    // Half register
    #[case(4, 4, vec![x0], "e00740b9e0cf1fb8")] // Single register.
    #[case(4, 4, vec![x0, x1], "e00740b9e0cf1fb8")] // Two registers
    #[case(4, 4, vec![x0, v0, v1], "e00740b9e0cf1fb8")] // Two vectors + reg

    // Full register
    #[case(8, 8, vec![x0], "e00740f9e08f1ff8")] // Single register.
    #[case(8, 8, vec![x0, x1], "e00740f9e08f1ff8")] // Two registers
    #[case(8, 8, vec![x0, v0, v1], "e00740f9e08f1ff8")] // Two vectors + reg

    // Two full registers
    #[case(16, 16, vec![x0], "e00b40f9e08f1ff8e00b40f9e08f1ff8")] // Single register.
    #[case(16, 16, vec![x0, x1], "e00741a9e007bfa9")] // Two registers
    #[case(16, 16, vec![x0, v0, v1], "e007c03de00f9f3c")] // Two vectors + reg

    // Two vectors
    #[case(32, 32, vec![x0], "e01340f9e08f1ff8e01340f9e08f1ff8e01340f9e08f1ff8e01340f9e08f1ff8")] // Single register.
    #[case(32, 32, vec![x0, x1], "e00742a9e007bfa9e00742a9e007bfa9")] // Two registers
    #[case(32, 32, vec![x0, v0, v1], "e00741ade007bfad")] // Two vectors + reg

    // High Reg Test
    #[case(32, 32, vec![x28], "fc1340f9fc8f1ff8fc1340f9fc8f1ff8fc1340f9fc8f1ff8fc1340f9fc8f1ff8")] // Single register.
    #[case(32, 32, vec![x28, x29], "fc7742a9fc77bfa9fc7742a9fc77bfa9")] // Two registers
    #[case(32, 32, vec![x28, v28, v29], "fc7741adfc77bfad")] // Two vectors + reg
    fn standard_cases(
        #[case] offset: i32,
        #[case] item_size: u32,
        #[case] scratch: Vec<AllRegisters>,
        #[case] expected_hex: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let rc_vec = Rc::new(scratch);
        let operation = PushStack::new(offset, item_size, rc_vec);

        assert!(encode_push_stack(&operation, &mut pc, &mut buf).is_ok());
        assert_encode(expected_hex, &buf, pc);
    }
}
