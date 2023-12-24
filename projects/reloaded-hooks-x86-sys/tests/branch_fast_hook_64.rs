#![allow(clippy::useless_transmute)]
#![allow(clippy::transmute_null_to_fn)]
#![allow(invalid_value)]

mod asm;

#[cfg(target_arch = "x86_64")]
mod tests {
    use crate::asm;
    use crate::asm::assemble_function::alloc_function_with_settings;
    use crate::asm::calculator::Add;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_MSFT_X64;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_MSFT_X64_CALL_OFFSET;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_MSFT_X64_TARGET_FUNCTION_OFFSET;
    use asm::assemble_function::alloc_function;
    use core::mem::transmute;
    use reloaded_hooks_portable::api::buffers::default_buffer::LockedBuffer;
    use reloaded_hooks_portable::api::buffers::default_buffer_factory::DefaultBufferFactory;
    use reloaded_hooks_portable::api::hooks::branch::branch_hook_fast::create_branch_hook_with_pointer;
    use reloaded_hooks_portable::api::settings::basic_hook_settings::BasicHookSettings;
    use reloaded_hooks_x86_sys::x64::{
        self, jit::JitX64, length_disassembler::LengthDisassemblerX64, rewriter::CodeRewriterX64,
    };

    // https://doc.rust-lang.org/std/option/index.html#representation
    pub static mut MAIN_TEST_ADDR: Option<Add> = None;
    pub unsafe extern "win64" fn add_hook_impl(x: i64, y: i64) -> i64 {
        MAIN_TEST_ADDR.unwrap_unchecked()(x + 1, y)
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn hook_calculator_branch_fast_x64_with_call_abs() {
        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD_MSFT_X64).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_MSFT_X64_CALL_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_MSFT_X64_CALL_OFFSET;
            let hook_target: usize = add_hook_impl as *const () as usize;

            let settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(x64::Register::r8),
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            create_branch_hook_with_pointer::<
                JitX64,
                x64::Register,
                LengthDisassemblerX64,
                CodeRewriterX64,
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
    #[cfg(target_arch = "x86_64")]
    fn hook_calculator_branch_fast_x64_with_rel_branch() {
        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD_MSFT_X64).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_MSFT_X64_CALL_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_MSFT_X64_CALL_OFFSET;
            let hook_target: usize =
                transmute(add_addr + CALL_CALCULATOR_ADD_MSFT_X64_TARGET_FUNCTION_OFFSET);

            let settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(x64::Register::r8),
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            create_branch_hook_with_pointer::<
                JitX64,
                x64::Register,
                LengthDisassemblerX64,
                CodeRewriterX64,
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
