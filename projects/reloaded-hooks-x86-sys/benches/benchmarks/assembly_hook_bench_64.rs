use core::mem;

use super::helpers::{alloc_function, CALCULATOR_ADD_CDECL_X86};
use criterion::Criterion;
use reloaded_hooks_buffers_common::buffer::StaticLinkedBuffer;
use reloaded_hooks_buffers_common::buffer_factory::BuffersFactory;
use reloaded_hooks_portable::api::buffers::buffer_abstractions::BufferFactory;
use reloaded_hooks_portable::api::hooks::assembly::assembly_hook::AssemblyHook;
use reloaded_hooks_portable::api::jit::compiler::Jit;
use reloaded_hooks_portable::api::settings::assembly_hook_settings::AssemblyHookSettings;
use reloaded_hooks_portable::api::settings::proximity_target::ProximityTarget;
use reloaded_hooks_x86_sys::x86;
use reloaded_hooks_x86_sys::x86::jit::JitX86;
use reloaded_hooks_x86_sys::x86::length_disassembler::LengthDisassemblerX86;
use reloaded_hooks_x86_sys::x86::rewriter::CodeRewriterX86;

// Assembler benches.
#[allow(dead_code)]
pub(crate) fn benchmark_create_assembly_hook(c: &mut Criterion) {
    // Allocate the function.
    let add_addr = alloc_function(&CALCULATOR_ADD_CDECL_X86).unwrap();

    // Preallocate a 256MB buffer (to make this test less accounting of unrealistic buffer allocations)
    // We want to measure the time it takes to create the hook, not the time it takes to allocate unrealistic amount of buffers.
    let proximity_target = ProximityTarget::new(add_addr, 0x8000000 * 2, 0x80000000);
    let _buf_opt = BuffersFactory::get_buffer(
        0x8000000 * 2,
        proximity_target.target_address,
        proximity_target.requested_proximity,
        JitX86::code_alignment(),
    );
    mem::drop(_buf_opt); // return the buffer
    let code = &[0xffu8, 0x44, 0x24, 0x08]; // inc dword ptr [esp + 4]
    let settings: AssemblyHookSettings<x86::Register> =
        AssemblyHookSettings::new_minimal(add_addr, code.as_ptr() as usize, code.len(), 6)
            .with_scratch_register(x86::Register::ecx);

    c.bench_function("assembly_hook_creation", |b| {
        b.iter(|| {
            let _hook = unsafe {
                AssemblyHook::<
                    StaticLinkedBuffer,
                    JitX86,
                    x86::Register,
                    LengthDisassemblerX86,
                    CodeRewriterX86,
                    BuffersFactory,
                >::create(&settings)
                .unwrap()
            };
        });
    });
}
