mod asm;

#[cfg(target_arch = "x86")]
mod tests {
    use crate::asm;
    use asm::assemble_function::alloc_function;
    use asm::calculator::{Add, CALCULATOR_ADD_CDECL_X86};
    use core::mem::transmute;
    use reloaded_hooks_portable::api::buffers::default_buffer::LockedBuffer;
    use reloaded_hooks_portable::api::hooks::assembly::assembly_hook::create_assembly_hook;
    use reloaded_hooks_portable::api::{
        buffers::default_buffer_factory::DefaultBufferFactory,
        settings::assembly_hook_settings::AssemblyHookSettings,
    };
    use reloaded_hooks_x86_sys::x86;
    use reloaded_hooks_x86_sys::x86::jit::JitX86;
    use reloaded_hooks_x86_sys::x86::length_disassembler::LengthDisassemblerX86;
    use reloaded_hooks_x86_sys::x86::rewriter::CodeRewriterX86;

    #[test]
    fn hook_calculator_add_asm_x86() {
        // Allocate the function.

        let add_addr = alloc_function(&CALCULATOR_ADD_CDECL_X86).unwrap();
        let add: Add = unsafe { transmute(add_addr) };

        // Overwrite the first bytes with hook
        let code = &[0xffu8, 0x44, 0x24, 0x08]; // inc dword ptr [esp + 4]
        let _hook = unsafe {
            let settings =
                AssemblyHookSettings::new_minimal(add_addr, code.as_ptr() as usize, code.len(), 6)
                    .with_scratch_register(x86::Register::ecx);

            create_assembly_hook::<
                JitX86,
                x86::Register,
                LengthDisassemblerX86,
                CodeRewriterX86,
                LockedBuffer,
                DefaultBufferFactory,
            >(&settings)
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
    fn hook_calculator_enable_disable_x86() {
        // Allocate the function.
        let add_addr = alloc_function(&CALCULATOR_ADD_CDECL_X86).unwrap();
        let add: Add = unsafe { transmute(add_addr) };

        // Overwrite the first bytes with hook
        let code = &[0xffu8, 0x44, 0x24, 0x08]; // inc dword ptr [esp + 4]
        let _hook = unsafe {
            let settings =
                AssemblyHookSettings::new_minimal(add_addr, code.as_ptr() as usize, code.len(), 6)
                    .with_scratch_register(x86::Register::ecx);

            create_assembly_hook::<
                JitX86,
                x86::Register,
                LengthDisassemblerX86,
                CodeRewriterX86,
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
    fn double_hook_calculator_add_asm_x86() {
        // Allocate the function.
        let add_addr = alloc_function(&CALCULATOR_ADD_CDECL_X86).unwrap();
        let add: Add = unsafe { transmute(add_addr) };

        // Overwrite the first bytes with hook
        let code = &[0xffu8, 0x44, 0x24, 0x08]; // inc dword ptr [esp + 4]
        let settings =
            AssemblyHookSettings::new_minimal(add_addr, code.as_ptr() as usize, code.len(), 6)
                .with_scratch_register(x86::Register::ecx);

        let _hook = unsafe {
            create_assembly_hook::<
                JitX86,
                x86::Register,
                LengthDisassemblerX86,
                CodeRewriterX86,
                LockedBuffer,
                DefaultBufferFactory,
            >(&settings)
            .unwrap()
        };

        let _hook2 = unsafe {
            create_assembly_hook::<
                JitX86,
                x86::Register,
                LengthDisassemblerX86,
                CodeRewriterX86,
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
