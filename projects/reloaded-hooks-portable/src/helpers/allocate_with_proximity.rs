extern crate alloc;

use crate::api::{
    buffers::buffer_abstractions::{Buffer, BufferFactory},
    jit::compiler::Jit,
    platforms::platform_functions::get_factory,
    settings::proximity_target::ProximityTarget,
    traits::register_info::RegisterInfo,
};
use alloc::boxed::Box;

pub(crate) fn get_buffer_from_factory<TJit, TRegister>(
    target_address: usize,
    target_size: u32,
) -> (bool, Box<dyn Buffer>)
where
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo,
{
    let buffer_factory: &mut Box<dyn BufferFactory> = get_factory();

    // Try known relative jump ranges.
    for &requested_proximity in TJit::max_relative_jump_distances() {
        let proximity_target =
            ProximityTarget::new(target_address, target_size, requested_proximity);

        let buf_opt = buffer_factory.get_buffer(
            target_size,
            proximity_target.target_address,
            proximity_target.requested_proximity,
            <TJit as Jit<TRegister>>::code_alignment(),
        );

        if let Ok(buffer) = buf_opt {
            return (true, buffer);
        }
    }

    let buf_boxed = buffer_factory
        .get_any_buffer(target_size, <TJit as Jit<TRegister>>::code_alignment())
        .unwrap();
    (false, buf_boxed)
}
