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
    use asm::assemble_function::alloc_function;
    use core::mem::transmute;
    use reloaded_hooks_aarch64_sys::all_registers::AllRegisters;
    use reloaded_hooks_aarch64_sys::calling_convention::CallingConvention;
    use reloaded_hooks_aarch64_sys::jit::JitAarch64;
    use reloaded_hooks_aarch64_sys::length_disassembler::LengthDisassemblerAarch64;
    use reloaded_hooks_aarch64_sys::rewriter::CodeRewriterAarch64;
    use reloaded_hooks_buffers_common::buffer::StaticLinkedBuffer;
    use reloaded_hooks_buffers_common::buffer_factory::BuffersFactory;
    use reloaded_hooks_portable::api::calling_convention_info::GenericCallingConvention;
    use reloaded_hooks_portable::api::function_info::BasicFunctionInfo;
    use reloaded_hooks_portable::api::function_info::ParameterType;
    use reloaded_hooks_portable::api::hooks::branch::branch_hook::create_branch_hook_with_pointer;
    use reloaded_hooks_portable::api::settings::basic_hook_settings::BasicHookSettings;
    use reloaded_hooks_portable::api::settings::function_hook_settings::FunctionHookSettings;

    // https://doc.rust-lang.org/std/option/index.html#representation
    pub static mut MAIN_TEST_ADDR: Option<Add> = None;

    // Exports as aapcs64, at least on aarch64-unknown-linux-gnu
    pub unsafe extern "C" fn add_hook_impl_aapcs(x: i64, y: i64) -> i64 {
        MAIN_TEST_ADDR.unwrap_unchecked()(x + 1, y)
    }

    // Static instance of BasicFunctionInfo
    static ADD_INFO: BasicFunctionInfo =
        BasicFunctionInfo::new(&[ParameterType::i64, ParameterType::i64]);

    // TODO: Tests for non-standard conventions. Unfortunately rust compiler doesn't support any.

    #[test]
    fn hook_calculator_branch_aarch64_with_enable_disable() {
        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_FUN_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_CALL_OFFSET;
            let hook_target: usize = add_hook_impl_aapcs as *const () as usize;

            let basic_settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(AllRegisters::x7),
            );

            let settings = FunctionHookSettings::<
                AllRegisters,
                BasicFunctionInfo,
                GenericCallingConvention<AllRegisters>,
            >::new(
                basic_settings,
                true,
                ADD_INFO,
                CallingConvention::aapcs64(),
                CallingConvention::aapcs64(),
                None,
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            let _hook = create_branch_hook_with_pointer::<
                JitAarch64,
                AllRegisters,
                LengthDisassemblerAarch64,
                CodeRewriterAarch64,
                StaticLinkedBuffer,
                BuffersFactory,
                BasicFunctionInfo,
                GenericCallingConvention<AllRegisters>,
            >(&settings, test_addr_ptr)
            .unwrap();

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
    }
}
