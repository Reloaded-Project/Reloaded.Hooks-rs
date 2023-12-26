#![allow(clippy::useless_transmute)]
#![allow(clippy::transmute_null_to_fn)]
#![allow(invalid_value)]

mod asm;

#[cfg(target_arch = "x86_64")]
mod tests {
    use crate::asm;
    use crate::asm::calculator::Add;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_MSFT_X64;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_MSFT_X64_CALL_OFFSET;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_MSFT_X64_FUN_OFFSET;
    use asm::assemble_function::alloc_function;
    use core::mem::transmute;
    use reloaded_hooks_buffers_common::buffer::StaticLinkedBuffer;
    use reloaded_hooks_buffers_common::buffer_factory::BuffersFactory;
    use reloaded_hooks_portable::api::calling_convention_info::GenericCallingConvention;
    use reloaded_hooks_portable::api::function_info::BasicFunctionInfo;
    use reloaded_hooks_portable::api::function_info::ParameterType;
    use reloaded_hooks_portable::api::hooks::branch::branch_hook::create_branch_hook_with_pointer;
    use reloaded_hooks_portable::api::settings::basic_hook_settings::BasicHookSettings;
    use reloaded_hooks_portable::api::settings::function_hook_settings::FunctionHookSettings;
    use reloaded_hooks_x86_sys::x64::calling_convention::CallingConvention;
    use reloaded_hooks_x86_sys::x64::Register;
    use reloaded_hooks_x86_sys::x64::{
        self, jit::JitX64, length_disassembler::LengthDisassemblerX64, rewriter::CodeRewriterX64,
    };

    // https://doc.rust-lang.org/std/option/index.html#representation
    pub static mut MAIN_TEST_ADDR: Option<Add> = None;

    // System V AMD64 calling convention!!
    pub unsafe extern "sysv64" fn add_hook_impl_sysv(x: i64, y: i64) -> i64 {
        MAIN_TEST_ADDR.unwrap_unchecked()(x + 1, y)
    }

    pub unsafe extern "win64" fn add_hook_impl_msft(x: i64, y: i64) -> i64 {
        MAIN_TEST_ADDR.unwrap_unchecked()(x + 1, y)
    }

    // Static instance of BasicFunctionInfo
    static ADD_INFO: BasicFunctionInfo =
        BasicFunctionInfo::new(&[ParameterType::i64, ParameterType::i64]);

    //#[test]
    fn hook_calculator_branch_x64_with_calling_convention_conversion() {
        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD_MSFT_X64).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_MSFT_X64_FUN_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_MSFT_X64_CALL_OFFSET;
            let hook_target: usize = add_hook_impl_sysv as *const () as usize;

            let basic_settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(x64::Register::r8),
            );

            let settings = FunctionHookSettings::<
                Register,
                BasicFunctionInfo,
                GenericCallingConvention<Register>,
            >::new(
                basic_settings,
                true,
                ADD_INFO,
                CallingConvention::microsoft_x64(),
                CallingConvention::system_v_amd64(),
                None,
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            create_branch_hook_with_pointer::<
                JitX64,
                x64::Register,
                LengthDisassemblerX64,
                CodeRewriterX64,
                StaticLinkedBuffer,
                BuffersFactory,
                BasicFunctionInfo,
                GenericCallingConvention<Register>,
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
    fn hook_calculator_branch_x64_with_enable_disable() {
        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD_MSFT_X64).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_MSFT_X64_FUN_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_MSFT_X64_CALL_OFFSET;
            let hook_target: usize = add_hook_impl_msft as *const () as usize;

            let basic_settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(x64::Register::r8),
            );

            let settings = FunctionHookSettings::<
                Register,
                BasicFunctionInfo,
                GenericCallingConvention<Register>,
            >::new(
                basic_settings,
                true,
                ADD_INFO,
                CallingConvention::microsoft_x64(),
                CallingConvention::microsoft_x64(),
                None,
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            let _hook = create_branch_hook_with_pointer::<
                JitX64,
                x64::Register,
                LengthDisassemblerX64,
                CodeRewriterX64,
                StaticLinkedBuffer,
                BuffersFactory,
                BasicFunctionInfo,
                GenericCallingConvention<Register>,
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
