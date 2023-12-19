use core::mem;

use super::helpers::{alloc_function, CALCULATOR_ADD_MSFT_X64};
use criterion::Criterion;
use reloaded_hooks_buffers_common::buffer::StaticLinkedBuffer;
use reloaded_hooks_buffers_common::buffer_factory::BuffersFactory;
use reloaded_hooks_portable::api::buffers::buffer_abstractions::BufferFactory;
use reloaded_hooks_portable::api::hooks::assembly::assembly_hook::AssemblyHook;
use reloaded_hooks_portable::api::jit::compiler::Jit;
use reloaded_hooks_portable::api::settings::assembly_hook_settings::AssemblyHookSettings;
use reloaded_hooks_portable::api::settings::proximity_target::ProximityTarget;
use reloaded_hooks_x86_sys::x64;
use reloaded_hooks_x86_sys::x64::{
    jit::JitX64, length_disassembler::LengthDisassemblerX64, rewriter::CodeRewriterX64,
};

// Assembler benches.
pub(crate) fn benchmark_create_assembly_hook(c: &mut Criterion) {
    // Allocate the function.
    let add_addr = alloc_function(&CALCULATOR_ADD_MSFT_X64).unwrap();

    // Preallocate a 8MB buffer (to make this test less accounting of unrealistic buffer allocations)
    // We want to measure the time it takes to create the hook, not the time it takes to allocate unrealistic amount of buffers.
    let proximity_target = ProximityTarget::new(add_addr, 0x800000, 0x80000000);
    let _buf_opt = BuffersFactory::get_buffer(
        0x800000,
        proximity_target.target_address,
        proximity_target.requested_proximity,
        JitX64::code_alignment(),
    );
    mem::drop(_buf_opt); // return the buffer

    let settings: AssemblyHookSettings<'_, x64::Register> = AssemblyHookSettings::new_minimal(
        add_addr,
        &[0x48, 0xFF, 0xC1], // inc rcx
        13,
    )
    .with_scratch_register(x64::Register::r8);

    c.bench_function("assembly_hook_creation", |b| {
        b.iter(|| {
            let _hook = unsafe {
                AssemblyHook::<
                    StaticLinkedBuffer,
                    JitX64,
                    x64::Register,
                    LengthDisassemblerX64,
                    CodeRewriterX64,
                    BuffersFactory,
                >::create(&settings)
                .unwrap()
            };
        });
    });
}
