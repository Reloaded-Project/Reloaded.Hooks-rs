#![allow(clippy::useless_transmute)]
#![allow(clippy::transmute_null_to_fn)]
#![allow(invalid_value)]

mod asm;

#[cfg(target_arch = "x86")]
mod tests {
    use crate::asm;
    use crate::asm::calculator::Add;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_CDECL_X86;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_CDECL_X86_CALL_OFFSET;
    use crate::asm::calculator::CALL_CALCULATOR_ADD_CDECL_X86_FUN_OFFSET;
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
    use reloaded_hooks_x86_sys::x86;
    use reloaded_hooks_x86_sys::x86::calling_convention::CallingConvention;
    use reloaded_hooks_x86_sys::x86::jit::JitX86;
    use reloaded_hooks_x86_sys::x86::length_disassembler::LengthDisassemblerX86;
    use reloaded_hooks_x86_sys::x86::rewriter::CodeRewriterX86;
    use reloaded_hooks_x86_sys::x86::Register;

    // https://doc.rust-lang.org/std/option/index.html#representation
    pub static mut MAIN_TEST_ADDR: Option<Add> = None;

    // (Microsoft) Thiscall
    pub unsafe extern "thiscall" fn add_hook_impl_thiscall(x: i32, y: i32) -> i32 {
        MAIN_TEST_ADDR.unwrap_unchecked()(x + 1, y)
    }

    pub unsafe extern "cdecl" fn add_hook_impl_cdecl(x: i32, y: i32) -> i32 {
        MAIN_TEST_ADDR.unwrap_unchecked()(x + 1, y)
    }

    // Static instance of BasicFunctionInfo
    static ADD_INFO: BasicFunctionInfo =
        BasicFunctionInfo::new(&[ParameterType::i32, ParameterType::i32]);

    #[test]
    fn hook_calculator_branch_x86_with_calling_convention_conversion() {
        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD_CDECL_X86).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_CDECL_X86_FUN_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_CDECL_X86_CALL_OFFSET;
            let hook_target: usize = add_hook_impl_thiscall as *const () as usize;

            let basic_settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(x86::Register::ecx),
            );

            let settings = FunctionHookSettings::<
                Register,
                BasicFunctionInfo,
                GenericCallingConvention<Register>,
            >::new(
                basic_settings,
                true,
                ADD_INFO,
                CallingConvention::cdecl(),
                CallingConvention::microsoft_thiscall(),
                None,
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            create_branch_hook_with_pointer::<
                JitX86,
                x86::Register,
                LengthDisassemblerX86,
                CodeRewriterX86,
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
    fn hook_calculator_branch_x86_with_enable_disable() {
        unsafe {
            // Allocate the function.
            let add_addr = alloc_function(&CALL_CALCULATOR_ADD_CDECL_X86).unwrap();
            let add: Add = transmute(add_addr + CALL_CALCULATOR_ADD_CDECL_X86_FUN_OFFSET);
            let hook_addr = add_addr + CALL_CALCULATOR_ADD_CDECL_X86_CALL_OFFSET;
            let hook_target: usize = add_hook_impl_cdecl as *const () as usize;

            let basic_settings = BasicHookSettings::new_with_scratch_register(
                hook_addr,
                hook_target,
                Some(x86::Register::ecx),
            );

            let settings = FunctionHookSettings::<
                Register,
                BasicFunctionInfo,
                GenericCallingConvention<Register>,
            >::new(
                basic_settings,
                true,
                ADD_INFO,
                CallingConvention::cdecl(),
                CallingConvention::cdecl(),
                None,
            );

            let test_addr_ptr: *mut usize = transmute(&MAIN_TEST_ADDR);
            let _hook = create_branch_hook_with_pointer::<
                JitX86,
                x86::Register,
                LengthDisassemblerX86,
                CodeRewriterX86,
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
