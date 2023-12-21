extern crate alloc;
use core::mem::size_of;

use crate::api::{
    buffers::buffer_abstractions::{Buffer, BufferFactory},
    jit::{
        compiler::{Jit, JitCapabilities, JitError},
        operation::Operation,
        operation_aliases::{JumpAbs, JumpAbsInd, JumpRel},
    },
    traits::register_info::RegisterInfo,
};
use alloc::string::ToString;
use alloc::vec::Vec;

/// Creates a jump operation for a given address.
///
/// # Parameters
/// - `mem_address` - The address of where the jump operation should be emplaced.
/// - `can_relative_jump` - True if the jump should be relative, else false.
/// - `target` - The address of the target to jump to.
/// - `scratch_register` - The scratch register to use for the jump on platforms that require it.
pub(crate) fn create_jump_operation<TRegister, TJit, TBufferFactory, TBuffer>(
    mem_address: usize,
    can_relative_jump: bool,
    target: usize,
    scratch_register: Option<TRegister>,
    buffer: &mut Vec<u8>,
) -> Result<(), JitError<TRegister>>
where
    TRegister: RegisterInfo + Clone + Default,
    TJit: Jit<TRegister>,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
{
    // Run accelerated command if possible as full JIT'ter can sometimes be slow.
    if can_relative_jump {
        let mut mem_addr = mem_address;
        TJit::encode_jump(&JumpRel::new(target), &mut mem_addr, buffer)?;
        return Ok(());
    }

    let ops: [Operation<TRegister>; 1] = create_jump_operation_ops::<
        TRegister,
        TJit,
        TBufferFactory,
        TBuffer,
    >(can_relative_jump, target, scratch_register)?;
    _ = TJit::compile_with_buf(mem_address, &ops, buffer);
    Ok(())
}

/// Creates a jump operation for a given address.
///
/// # Parameters
/// - `can_relative_jump` - True if the jump should be relative, else false.
/// - `target` - The address of the target to jump to.
/// - `scratch_register` - The scratch register to use for the jump on platforms that require it.
pub(crate) fn create_jump_operation_ops<TRegister, TJit, TBufferFactory, TBuffer>(
    can_relative_jump: bool,
    target: usize,
    scratch_register: Option<TRegister>,
) -> Result<[Operation<TRegister>; 1], JitError<TRegister>>
where
    TRegister: RegisterInfo + Clone + Default,
    TJit: Jit<TRegister>,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
{
    if can_relative_jump {
        return Ok([Operation::JumpRelative(JumpRel::new(target))]);
    }

    // Try to use absolute indirect jump
    if TJit::get_jit_capabilities().contains(JitCapabilities::PROFITABLE_ABSOLUTE_INDIRECT_JUMP) {
        for offset in TJit::max_indirect_offsets() {
            let buf = TBufferFactory::get_buffer(
                size_of::<usize>() as u32,
                (offset / 2) as usize,
                (offset / 2 - 1) as usize,
                size_of::<usize>() as u32,
            );

            match buf {
                Err(_err) => continue,
                Ok(mut ok) => {
                    let addr = ok.get_address();
                    let bytes = target.to_ne_bytes();
                    ok.write(&bytes);
                    return Ok([Operation::JumpAbsoluteIndirect(JumpAbsInd {
                        pointer_address: addr as usize,
                        scratch_register,
                    })]);
                }
            }
        }
    }

    // Otherwise absolute jump with scratch
    if scratch_register.is_none() {
        return Err(JitError::NoScratchRegister(
            "Needed for create_jump_operation_ops".to_string(),
        ));
    }

    Ok([Operation::JumpAbsolute(JumpAbs {
        target_address: target,
        scratch_register: scratch_register.unwrap(),
    })])
}
