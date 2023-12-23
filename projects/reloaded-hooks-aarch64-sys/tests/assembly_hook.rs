mod asm;

#[cfg(target_arch = "aarch64")]
mod tests {
    use crate::asm;
    use asm::assemble_function::alloc_function;
    use asm::calculator::{Add, CALCULATOR_ADD};
    use core::mem::transmute;
    use reloaded_hooks_aarch64_sys::all_registers::AllRegisters;
    use reloaded_hooks_aarch64_sys::jit::JitAarch64;
    use reloaded_hooks_aarch64_sys::length_disassembler::LengthDisassemblerAarch64;
    use reloaded_hooks_aarch64_sys::rewriter::CodeRewriterAarch64;
    use reloaded_hooks_portable::api::buffers::default_buffer::LockedBuffer;

    use reloaded_hooks_portable::api::{
        buffers::default_buffer_factory::DefaultBufferFactory,
        hooks::assembly::assembly_hook::AssemblyHook,
        settings::assembly_hook_settings::AssemblyHookSettings,
    };

    #[test]
    fn hook_calculator_add_asm() {
        // Allocate the function.
        let add_addr = alloc_function(&CALCULATOR_ADD).unwrap();
        let add: Add = unsafe { transmute(add_addr) };

        // Overwrite the first bytes with hook
        let code = &[0x21u8, 0x04, 0x00, 0x91];
        let _hook = unsafe {
            let settings =
                AssemblyHookSettings::new_minimal(add_addr, code.as_ptr() as usize, code.len(), 20)
                    .with_scratch_register(AllRegisters::x7);

            AssemblyHook::<
                LockedBuffer,
                JitAarch64,
                AllRegisters,
                LengthDisassemblerAarch64,
                CodeRewriterAarch64,
                DefaultBufferFactory,
            >::create(&settings)
            .unwrap()
        };

        // Make the hook
        for x in 0..100 {
            for y in 0..100 {
                let expected = x + y + 1;
                let actual = add(x, y);
                assert_eq!(expected, actual);
            }
        }
    }

    #[test]
    fn hook_calculator_enable_disable() {
        // Allocate the function.
        let add_addr = alloc_function(&CALCULATOR_ADD).unwrap();
        let add: Add = unsafe { transmute(add_addr) };

        // Overwrite the first bytes with hook
        let code = &[0x21u8, 0x04, 0x00, 0x91];
        let _hook = unsafe {
            let settings =
                AssemblyHookSettings::new_minimal(add_addr, code.as_ptr() as usize, code.len(), 20)
                    .with_scratch_register(AllRegisters::x7);

            AssemblyHook::<
                LockedBuffer,
                JitAarch64,
                AllRegisters,
                LengthDisassemblerAarch64,
                CodeRewriterAarch64,
                DefaultBufferFactory,
            >::create(&settings)
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
    fn double_hook_calculator_add_asm() {
        // Allocate the function.
        let add_addr = alloc_function(&CALCULATOR_ADD).unwrap();
        let add: Add = unsafe { transmute(add_addr) };

        // Overwrite the first bytes with hook
        let code = &[0x21u8, 0x04, 0x00, 0x91];
        let settings =
            AssemblyHookSettings::new_minimal(add_addr, code.as_ptr() as usize, code.len(), 20)
                .with_scratch_register(AllRegisters::x7);

        let _hook = unsafe {
            AssemblyHook::<
                LockedBuffer,
                JitAarch64,
                AllRegisters,
                LengthDisassemblerAarch64,
                CodeRewriterAarch64,
                DefaultBufferFactory,
            >::create(&settings)
            .unwrap()
        };

        let _hook2 = unsafe {
            AssemblyHook::<
                LockedBuffer,
                JitAarch64,
                AllRegisters,
                LengthDisassemblerAarch64,
                CodeRewriterAarch64,
                DefaultBufferFactory,
            >::create(&settings)
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
