#![allow(clippy::useless_transmute)]
#![allow(clippy::transmute_null_to_fn)]
#![allow(invalid_value)]

mod asm;

#[cfg(target_arch = "x86")]
mod tests {
    use crate::asm;
    use crate::asm::assemble_function::alloc_function_with_settings;
    use crate::asm::calculator::Add;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_CDECL_X86;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_CDECL_X86_CALL_OFFSET;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_CDECL_X86_FUN_OFFSET;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_CDECL_X86_TARGET_FUNCTION_OFFSET;
    use asm::assemble_function::alloc_function;
    use core::mem::transmute;
    use reloaded_hooks_portable::api::buffers::default_buffer::LockedBuffer;
    use reloaded_hooks_portable::api::buffers::default_buffer_factory::DefaultBufferFactory;
    use reloaded_hooks_portable::api::hooks::branch::branch_hook_fast::create_branch_hook_with_pointer;
    use reloaded_hooks_portable::api::settings::basic_hook_settings::BasicHookSettings;
    use reloaded_hooks_x86_sys::x86::{
        self, jit::JitX86, length_disassembler::LengthDisassemblerX86, rewriter::CodeRewriterX86,
    };

    // https://doc.rust-lang.org/std/option/index.html#representation
    pub static mut MAIN_TEST_ADDR: Option<Add> = None;
    pub unsafe extern "cdecl" fn add_hook_impl(x: i32, y: i32) -> i32 {
        MAIN_TEST_ADDR.unwrap_unchecked()(x + 1, y)
    }

    #[test]
    #[cfg(target_arch = "x86")]
    fn hook_calculator_branch_fast_x86_with_call_abs() {
        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD_CDECL_X86).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_CDECL_X86_FUN_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_CDECL_X86_CALL_OFFSET;
            let hook_target: usize = add_hook_impl as *const () as usize;

            let settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(x86::Register::ecx),
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            create_branch_hook_with_pointer::<
                JitX86,
                x86::Register,
                LengthDisassemblerX86,
                CodeRewriterX86,
                LockedBuffer,
                DefaultBufferFactory,
            >(&settings, test_addr_ptr)
            .unwrap();

            // Test the hook
            for x in 0..100 {
                for y in 0..100 {
                    assert_eq!(x + y + 1, add(x, y));
                }
            }
        }
    }

    #[test]
    #[cfg(target_arch = "x86")]
    fn hook_calculator_branch_fast_x86_with_rel_branch() {
        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD_CDECL_X86).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_CDECL_X86_FUN_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_CDECL_X86_CALL_OFFSET;
            let hook_target: usize =
                transmute(add_addr + CALL_CALCULATOR_ADD_CDECL_X86_TARGET_FUNCTION_OFFSET);

            let settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(x86::Register::ecx),
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            create_branch_hook_with_pointer::<
                JitX86,
                x86::Register,
                LengthDisassemblerX86,
                CodeRewriterX86,
                LockedBuffer,
                DefaultBufferFactory,
            >(&settings, test_addr_ptr)
            .unwrap();

            // Test the hook
            for x in 0..100 {
                for y in 0..100 {
                    assert_eq!(x + y + 1, add(x, y));
                }
            }
        }
    }
}
