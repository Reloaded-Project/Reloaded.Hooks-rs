extern crate alloc;
use core::hint::unreachable_unchecked;

use crate::api::jit::mov_from_stack_operation::MovFromStackOperation;
use crate::api::jit::operation::Operation;
use crate::api::jit::operation_aliases::{MovToStack, StackAlloc};
use crate::api::traits::register_info::RegisterInfo;
use alloc::vec::Vec;
use smallvec::smallvec;
use smallvec::SmallVec;

/// Decomposes [`Push`] operations in a sequence of [`Operation`]s into separate
/// [`MovToStack`] and [`StackAlloc`] operations.
pub(crate) fn decompose_push_operations<TRegister: Copy + Clone + RegisterInfo>(
    operations: &mut Vec<Operation<TRegister>>,
    regular_reg_size_bytes: usize,
) {
    /*
    Explanation.

    Before:

        push rax
        push rbx

    Broken Down:

        sub rsp, 8
        mov qword ptr [rsp], rax
        sub rsp, 8
        mov qword ptr [rsp], rbx

    After:

        mov qword ptr [rsp - 8], rax
        mov qword ptr [rsp - 16], rbx
        sub rsp, 16

    Stack View

        rbx @ -0
        rax @ -8
    */

    // Temporary working buffer.
    let mut ops: SmallVec<[Operation<TRegister>; 16]> = smallvec![];
    let mut stack_size: isize = 0;

    let mut first_push_idx = usize::MAX;
    let mut read_idx = 0;

    while read_idx < operations.len() {
        match &mut operations[read_idx] {
            Operation::Push(push) if push.register.size_in_bytes() > regular_reg_size_bytes => {
                if first_push_idx == usize::MAX {
                    first_push_idx = read_idx; // cmov :p
                }

                stack_size -= push.register.size_in_bytes() as isize;
                ops.push(MovToStack::new(stack_size as i32, push.register).into());
                read_idx += 1;
            }
            _ => {
                if stack_size != 0 {
                    do_decompose_push(operations, &mut ops, stack_size, first_push_idx);
                }

                first_push_idx = usize::MAX;
                stack_size = 0;
                read_idx += 1;
            }
        }
    }

    if stack_size != 0 {
        do_decompose_push(operations, &mut ops, stack_size, first_push_idx);
    }
}

fn do_decompose_push<TRegister: Copy + Clone>(
    operations: &mut Vec<Operation<TRegister>>,
    ops: &mut SmallVec<[Operation<TRegister>; 16]>,
    stack_size: isize,
    first_push_idx: usize,
) {
    // Emit the 'stackalloc' into original sequence, and splice.
    let splice_end = ops.len() + first_push_idx;
    ops.push(StackAlloc::new(-stack_size as i32).into());

    // Splice new operations into original operations
    drop(operations.splice(first_push_idx..splice_end, ops.iter().cloned()));

    // Clear ops
    ops.clear();
}

/// Decomposes [`Pop`] operations in a sequence of [`Operation`]s into separate
/// [`MovFromStack`] and [`StackAlloc`] operations.
pub(crate) fn decompose_pop_operations_ex<TRegister: Copy + Clone + RegisterInfo>(
    operations: &mut Vec<Operation<TRegister>>,
    regular_reg_size_bytes: usize,
) {
    /*
    Explanation.

    Before:

        pop rbx
        pop rax

    Broken Down:

        mov qword ptr [rsp], rbx
        sub rsp, -8
        mov qword ptr [rsp], rax
        sub rsp, -8

    After:

        mov qword ptr [rsp], rax
        mov qword ptr [rsp + 8], rbx
        sub rsp, -16

    Stack View

        rbx @ -0
        rax @ -8
    */

    /*
        Extra Functionality:

            Find any StackAlloc directly before the `pop` operation, and move to bottom of the sequence.
    */
    let mut ops: SmallVec<[Operation<TRegister>; 16]> = Default::default();
    let mut stack_size: isize = 0;

    let mut last_stackalloc_val = 0_i32;
    let mut last_stackalloc_idx = usize::MAX;
    let mut first_pop_idx = usize::MAX;
    let mut read_idx = 0;

    while read_idx < operations.len() {
        match operations[read_idx] {
            Operation::Pop(pop) if pop.register.size_in_bytes() > regular_reg_size_bytes => {
                if first_pop_idx == usize::MAX {
                    first_pop_idx = read_idx;
                }

                ops.push(MovFromStackOperation::new(stack_size as i32, pop.register).into());
                stack_size += pop.register.size_in_bytes() as isize;
                read_idx += 1;
            }
            Operation::StackAlloc(stackalloc) => {
                if last_stackalloc_idx != usize::MAX {
                    if stack_size != 0 {
                        do_decompose_pops(
                            operations,
                            &mut ops,
                            &mut stack_size,
                            last_stackalloc_val,
                            last_stackalloc_idx,
                            first_pop_idx,
                        );
                    }

                    first_pop_idx = usize::MAX;
                    stack_size = 0;
                }

                last_stackalloc_idx = read_idx;
                last_stackalloc_val = stackalloc.operand;
                read_idx += 1;
            }
            _ => {
                if stack_size != 0 {
                    do_decompose_pops(
                        operations,
                        &mut ops,
                        &mut stack_size,
                        last_stackalloc_val,
                        last_stackalloc_idx,
                        first_pop_idx,
                    );
                }

                first_pop_idx = usize::MAX;
                last_stackalloc_idx = usize::MAX;
                stack_size = 0;
                read_idx += 1;
            }
        }
    }

    if stack_size != 0 {
        do_decompose_pops(
            operations,
            &mut ops,
            &mut stack_size,
            last_stackalloc_val,
            last_stackalloc_idx,
            first_pop_idx,
        );
    }
}

fn do_decompose_pops<TRegister: Clone + Copy>(
    operations: &mut Vec<Operation<TRegister>>,
    ops: &mut SmallVec<[Operation<TRegister>; 16]>,
    stack_size: &mut isize,
    last_stackalloc_val: i32,
    last_stackalloc_idx: usize,
    first_pop_idx: usize,
) {
    let is_stackalloc = last_stackalloc_idx == first_pop_idx - 1;
    let stackalloc_offset = is_stackalloc as i32 * last_stackalloc_val;
    update_mov_from_stack_offsets(ops, last_stackalloc_val, is_stackalloc);

    // Emit stackalloc for the writes we just made.
    let stackalloc_ofs = -*stack_size as i32 + stackalloc_offset;
    let splice_end = ops.len() + first_pop_idx;
    ops.push(StackAlloc::new(stackalloc_ofs).into());

    // Splice new operations into original operations
    let splice_start = first_pop_idx - is_stackalloc as usize;
    drop(operations.splice(splice_start..splice_end, ops.iter().cloned()));

    // Clear ops
    ops.clear();
}

#[inline(always)]
fn update_mov_from_stack_offsets<TRegister: Copy + Clone>(
    ops: &mut SmallVec<[Operation<TRegister>; 16]>,
    last_stackalloc_val: i32,
    is_stackalloc: bool,
) {
    if is_stackalloc {
        for op in ops {
            if let Operation::MovFromStack(mov_op) = op {
                mov_op.stack_offset += last_stackalloc_val; // we add to positive offset
            } else {
                unsafe {
                    unreachable_unchecked(); // we only add MovFromStack operations at this point
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::jit::operation_aliases::{Pop, Push},
        helpers::test_helpers::MockRegister::{self, *},
    };

    #[test]
    fn decompose_push_nop() {
        let mut operations = vec![
            Operation::Push(Push::new(R1)),
            Operation::Push(Push::new(R2)),
            Operation::Pop(Pop::new(R1)), // End First Block
            Operation::Push(Push::new(R1)),
            Operation::Push(Push::new(R2)),
            Operation::Pop(Pop::new(R1)), // End Second Block
        ];

        decompose_push_operations::<MockRegister>(&mut operations, 4);

        assert_eq!(operations.len(), 6);
        assert_eq!(operations[0], Operation::Push(Push::new(R1)));
        assert_eq!(operations[1], Operation::Push(Push::new(R2)));
        assert_eq!(operations[2], Operation::Pop(Pop::new(R1)));
        assert_eq!(operations[3], Operation::Push(Push::new(R1)));
        assert_eq!(operations[4], Operation::Push(Push::new(R2)));
        assert_eq!(operations[5], Operation::Pop(Pop::new(R1)));
    }

    #[test]
    fn decompose_push_with_vector_regs() {
        let mut operations = vec![
            Operation::Push(Push::new(V1)),
            Operation::Push(Push::new(V2)),
            Operation::Pop(Pop::new(R1)), // End First Block
            Operation::Push(Push::new(V1)),
            Operation::Push(Push::new(V2)),
            Operation::Pop(Pop::new(R1)), // End Second Block
        ];

        decompose_push_operations::<MockRegister>(&mut operations, 4);

        assert_eq!(operations.len(), 8);
        assert_eq!(
            operations[0],
            Operation::MovToStack(MovToStack::new(-16, V1))
        );
        assert_eq!(
            operations[1],
            Operation::MovToStack(MovToStack::new(-32, V2))
        );
        assert_eq!(operations[2], Operation::StackAlloc(StackAlloc::new(32)));
        assert_eq!(operations[3], Operation::Pop(Pop::new(R1)));
        assert_eq!(
            operations[4],
            Operation::MovToStack(MovToStack::new(-16, V1))
        );
        assert_eq!(
            operations[5],
            Operation::MovToStack(MovToStack::new(-32, V2))
        );
        assert_eq!(operations[6], Operation::StackAlloc(StackAlloc::new(32)));
        assert_eq!(operations[7], Operation::Pop(Pop::new(R1)));
    }

    #[test]
    fn decompose_pop_nop() {
        let mut operations = vec![
            Operation::Push(Push::new(R1)), // Start First Block
            Operation::Pop(Pop::new(R1)),
            Operation::Pop(Pop::new(R2)),
            Operation::Push(Push::new(R1)), // Start Second Block
            Operation::Pop(Pop::new(R1)),
            Operation::Pop(Pop::new(R2)),
        ];

        decompose_pop_operations_ex::<MockRegister>(&mut operations, 4);

        assert_eq!(operations.len(), 6);
        assert_eq!(operations[0], Operation::Push(Push::new(R1)));
        assert_eq!(operations[1], Operation::Pop(Pop::new(R1)));
        assert_eq!(operations[2], Operation::Pop(Pop::new(R2)));
        assert_eq!(operations[3], Operation::Push(Push::new(R1)));
        assert_eq!(operations[4], Operation::Pop(Pop::new(R1)));
        assert_eq!(operations[5], Operation::Pop(Pop::new(R2)));
    }

    #[test]
    fn decompose_pop_with_vector_regs() {
        let mut operations = vec![
            Operation::Push(Push::new(R1)), // Start First Block
            Operation::Pop(Pop::new(V1)),
            Operation::Pop(Pop::new(V2)),
            Operation::Push(Push::new(R1)), // Start Second Block
            Operation::Pop(Pop::new(V1)),
            Operation::Pop(Pop::new(V2)),
        ];

        decompose_pop_operations_ex::<MockRegister>(&mut operations, 4);

        assert_eq!(operations.len(), 8);
        assert_eq!(operations[0], Operation::Push(Push::new(R1)));
        assert_eq!(
            operations[1],
            Operation::MovFromStack(MovFromStackOperation::new(0, V1))
        );
        assert_eq!(
            operations[2],
            Operation::MovFromStack(MovFromStackOperation::new(16, V2))
        );
        assert_eq!(operations[3], Operation::StackAlloc(StackAlloc::new(-32)));
        assert_eq!(operations[4], Operation::Push(Push::new(R1)));
        assert_eq!(
            operations[5],
            Operation::MovFromStack(MovFromStackOperation::new(0, V1))
        );
        assert_eq!(
            operations[6],
            Operation::MovFromStack(MovFromStackOperation::new(16, V2))
        );
        assert_eq!(operations[7], Operation::StackAlloc(StackAlloc::new(-32)));
    }

    #[test]
    fn decompose_pop_withstackalloc_andvectorregs() {
        let mut operations = vec![
            Operation::StackAlloc(StackAlloc::new(-32)), // Start First Block
            Operation::Pop(Pop::new(V1)),
            Operation::Pop(Pop::new(V2)),
            Operation::StackAlloc(StackAlloc::new(-32)), // Start Second Block
            Operation::Pop(Pop::new(V1)),
            Operation::Pop(Pop::new(V2)),
        ];

        decompose_pop_operations_ex::<MockRegister>(&mut operations, 4);

        assert_eq!(operations.len(), 6);
        assert_eq!(
            operations[0],
            Operation::MovFromStack(MovFromStackOperation::new(-32, V1))
        );
        assert_eq!(
            operations[1],
            Operation::MovFromStack(MovFromStackOperation::new(-16, V2))
        );
        assert_eq!(operations[2], Operation::StackAlloc(StackAlloc::new(-64)));
        assert_eq!(
            operations[3],
            Operation::MovFromStack(MovFromStackOperation::new(-32, V1))
        );
        assert_eq!(
            operations[4],
            Operation::MovFromStack(MovFromStackOperation::new(-16, V2))
        );
        assert_eq!(operations[5], Operation::StackAlloc(StackAlloc::new(-64)));
    }
}
