mod asm;

#[cfg(target_arch = "x86_64")]
mod tests {
    use crate::asm;
    use crate::asm::calculator::Add;
    use asm::assemble_function::alloc_function;
    use asm::calculator::CALCULATOR_ADD_MSFT_X64;
    use core::mem::transmute;
    use reloaded_hooks_portable::api::buffers::default_buffer::LockedBuffer;
    use reloaded_hooks_portable::api::hooks::assembly::assembly_hook::create_assembly_hook;
    use reloaded_hooks_portable::api::{
        buffers::default_buffer_factory::DefaultBufferFactory,
        settings::assembly_hook_settings::AssemblyHookSettings,
    };
    use reloaded_hooks_x86_sys::x64::{
        self, jit::JitX64, length_disassembler::LengthDisassemblerX64, rewriter::CodeRewriterX64,
    };

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn hook_calculator_add_asm_x64() {
        // Allocate the function.
        let add_addr = alloc_function(&CALCULATOR_ADD_MSFT_X64).unwrap();
        let add: Add = unsafe { transmute(add_addr) };

        // Overwrite the first bytes with hook
        let _hook = unsafe {
            let slice = &[0x48u8, 0xFF, 0xC1]; // inc rcx
            let settings = AssemblyHookSettings::new_minimal(
                add_addr,
                slice.as_ptr() as usize,
                slice.len(),
                13,
            )
            .with_scratch_register(x64::Register::r8);

            create_assembly_hook::<
                JitX64,
                x64::Register,
                LengthDisassemblerX64,
                CodeRewriterX64,
                LockedBuffer,
                DefaultBufferFactory,
            >(&settings)
            .unwrap()
        };

        // Make the hook
        for x in 0..100 {
            for y in 0..100 {
                assert_eq!(x + y + 1, add(x, y));
            }
        }
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn hook_calculator_enable_disable_x64() {
        // Allocate the function.
        let add_addr = alloc_function(&CALCULATOR_ADD_MSFT_X64).unwrap();
        let add: Add = unsafe { transmute(add_addr) };

        // Overwrite the first bytes with hook
        let _hook = unsafe {
            let slice = &[0x48u8, 0xFF, 0xC1]; // inc rcx
            let settings = AssemblyHookSettings::new_minimal(
                add_addr,
                slice.as_ptr() as usize,
                slice.len(),
                13,
            )
            .with_scratch_register(x64::Register::r8);

            create_assembly_hook::<
                JitX64,
                x64::Register,
                LengthDisassemblerX64,
                CodeRewriterX64,
                LockedBuffer,
                DefaultBufferFactory,
            >(&settings)
            .unwrap()
        };

        // Toggle enable/disable
        _hook.enable();
        for x in 0..100 {
            for y in 0..100 {
                assert_eq!(x + y + 1, add(x, y));
            }
        }

        _hook.disable();
        for x in 0..100 {
            for y in 0..100 {
                assert_eq!(x + y, add(x, y));
            }
        }

        _hook.enable();
        for x in 0..100 {
            for y in 0..100 {
                assert_eq!(x + y + 1, add(x, y));
            }
        }
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn double_hook_calculator_add_asm_x64() {
        // Allocate the function.
        let add_addr = alloc_function(&CALCULATOR_ADD_MSFT_X64).unwrap();
        let add: Add = unsafe { transmute(add_addr) };

        // Overwrite the first bytes with hook
        let slice = &[0x48u8, 0xFF, 0xC1]; // inc rcx
        let settings =
            AssemblyHookSettings::new_minimal(add_addr, slice.as_ptr() as usize, slice.len(), 13)
                .with_scratch_register(x64::Register::r8);

        let _hook = unsafe {
            create_assembly_hook::<
                JitX64,
                x64::Register,
                LengthDisassemblerX64,
                CodeRewriterX64,
                LockedBuffer,
                DefaultBufferFactory,
            >(&settings)
            .unwrap()
        };

        let _hook2 = unsafe {
            create_assembly_hook::<
                JitX64,
                x64::Register,
                LengthDisassemblerX64,
                CodeRewriterX64,
                LockedBuffer,
                DefaultBufferFactory,
            >(&settings)
            .unwrap()
        };

        // Make the hook
        for x in 0..100 {
            for y in 0..100 {
                assert_eq!(x + y + 2, add(x, y));
            }
        }
    }
}
