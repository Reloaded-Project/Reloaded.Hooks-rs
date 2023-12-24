#![allow(clippy::useless_transmute)]
#![allow(clippy::transmute_null_to_fn)]
#![allow(invalid_value)]

mod asm;

#[cfg(target_arch = "aarch64")]
mod tests {
    use crate::asm;
    use crate::asm::calculator::Add;
    use crate::asm::calculator::CALL_CALCULATOR_ADD;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_CALL_OFFSET;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_FUN_OFFSET;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_TARGET_FUNCTION_OFFSET;
    use asm::assemble_function::alloc_function;
    use core::mem::transmute;
    use reloaded_hooks_aarch64_sys::all_registers::AllRegisters;
    use reloaded_hooks_aarch64_sys::jit::JitAarch64;
    use reloaded_hooks_aarch64_sys::{
        length_disassembler::LengthDisassemblerAarch64, rewriter::CodeRewriterAarch64,
    };
    use reloaded_hooks_portable::api::buffers::default_buffer::LockedBuffer;
    use reloaded_hooks_portable::api::buffers::default_buffer_factory::DefaultBufferFactory;
    use reloaded_hooks_portable::api::hooks::branch::branch_hook_fast::create_branch_hook_fast_with_pointer;
    use reloaded_hooks_portable::api::settings::basic_hook_settings::BasicHookSettings;

    // https://doc.rust-lang.org/std/option/index.html#representation
    pub static mut MAIN_TEST_ADDR: Option<Add> = None;
    pub unsafe extern "C" fn add_hook_impl(x: i64, y: i64) -> i64 {
        MAIN_TEST_ADDR.unwrap_unchecked()(x + 1, y)
    }

    #[test]
    fn hook_calculator_branch_fast_aarch64_with_call_abs() {
        use reloaded_hooks_aarch64_sys::all_registers::AllRegisters;

        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_FUN_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_CALL_OFFSET;
            let hook_target: usize = add_hook_impl as *const () as usize;

            let settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(AllRegisters::x7),
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            create_branch_hook_fast_with_pointer::<
                JitAarch64,
                AllRegisters,
                LengthDisassemblerAarch64,
                CodeRewriterAarch64,
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
    fn hook_calculator_branch_fast_aarch64_with_rel_branch() {
        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_FUN_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_CALL_OFFSET;
            let hook_target: usize =
                transmute(add_addr + CALL_CALCULATOR_ADD_TARGET_FUNCTION_OFFSET);

            let settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(AllRegisters::x7),
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            create_branch_hook_fast_with_pointer::<
                JitAarch64,
                AllRegisters,
                LengthDisassemblerAarch64,
                CodeRewriterAarch64,
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
