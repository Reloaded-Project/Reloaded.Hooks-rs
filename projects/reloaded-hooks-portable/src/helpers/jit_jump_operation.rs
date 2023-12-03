extern crate alloc;
use crate::api::{
    jit::{
        compiler::{Jit, JitError},
        operation::Operation,
        operation_aliases::{JumpAbs, JumpRel},
    },
    traits::register_info::RegisterInfo,
};
use alloc::rc::Rc;

/// Creates a jump operation for a given address.
///
/// # Parameters
/// - `mem_address` - The address of where the jump operation should be emplaced.
/// - `can_relative_jump` - True if the jump should be relative, else false.
/// - `target` - The address of the target to jump to.
pub(crate) fn create_jump_operation<TRegister, TJit>(
    mem_address: usize,
    can_relative_jump: bool,
    target: usize,
    scratch_register: Option<TRegister>,
) -> Result<Rc<[u8]>, JitError<TRegister>>
where
    TRegister: RegisterInfo + Clone + Default,
    TJit: Jit<TRegister>, /* your trait bounds here, if needed */
{
    let ops: [Operation<TRegister>; 1] = if can_relative_jump {
        [Operation::JumpRelative(JumpRel::new(target))]
    } else {
        [Operation::JumpAbsolute(JumpAbs {
            target_address: target,
            scratch_register: scratch_register.unwrap_or_default(),
        })]
    };

    TJit::compile(mem_address, &ops)
}
