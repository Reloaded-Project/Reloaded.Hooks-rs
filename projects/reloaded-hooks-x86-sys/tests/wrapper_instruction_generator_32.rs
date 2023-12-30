#[cfg(test)]
pub mod tests {
    use core::mem::size_of;

    use reloaded_hooks_portable::api::calling_convention_info::{
        GenericCallingConvention, StackCleanup, StackParameterOrder,
    };
    use reloaded_hooks_portable::api::errors::wrapper_generation_error::WrapperGenerationError;
    use reloaded_hooks_portable::api::function_info::{BasicFunctionInfo, ParameterType};
    use reloaded_hooks_portable::api::jit::compiler::Jit;
    use reloaded_hooks_portable::api::jit::operation::Operation;
    use reloaded_hooks_portable::api::jit::operation::Operation::MultiPush;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use reloaded_hooks_portable::api::wrapper_instruction_generator::{
        generate_wrapper_instructions, WrapperInstructionGeneratorOptions,
    };
    use reloaded_hooks_x86_sys::x86::calling_convention::CallingConvention;
    use reloaded_hooks_x86_sys::x86::jit::JitX86;
    use reloaded_hooks_x86_sys::x86::Register::{self, *};
    use smallvec::smallvec;

    // EXTRA TESTS //

    #[test]
    fn ms_thiscall_to_cdecl_unoptimized_with_call_absolute() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters_with_address(
            CallingConvention::microsoft_thiscall(),
            CallingConvention::cdecl(),
            false,
            0xFFFFFFFF,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 5);
        assert_push_stack(&vec[0], nint, nint); // re-push right param
        assert_push_stack(&vec[1], nint * 3, nint); // re-push left param
        assert_eq!(vec[2], Pop::new(ecx).into()); // pop left param into reg
        assert_eq!(vec[3], CallAbs::new(0xFFFFFFFF).into());
        assert_eq!(vec[4], Return::new(0).into()); // caller cleanup, so no offset here
    }

    #[test]
    fn ms_thiscall_to_cdecl_unoptimized() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters(
            CallingConvention::microsoft_thiscall(),
            CallingConvention::cdecl(),
            false,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 5);

        assert_push_stack(&vec[0], nint, nint); // re-push right param
        assert_push_stack(&vec[1], nint * 3, nint); // re-push left param
        assert_eq!(vec[2], Pop::new(ecx).into()); // pop left param into reg
        assert_eq!(vec[3], CallRel::new(4096).into());
        assert_eq!(vec[4], Return::new(0).into()); // caller cleanup, so no offset here
    }

    #[test]
    fn ms_thiscall_to_cdecl_optimized() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters(
            CallingConvention::microsoft_thiscall(),
            CallingConvention::cdecl(),
            true,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 4);
        assert_push_stack(&vec[0], nint, nint); // re-push right param
        assert_eq!(vec[1], MovFromStack::new((nint * 3) as i32, ecx).into()); // mov left param to register
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], Return::new(0).into()); // caller cleanup, so no offset here
    }

    #[test]
    fn ms_cdecl_to_thiscall_unoptimized() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters(
            CallingConvention::cdecl(),
            CallingConvention::microsoft_thiscall(),
            false,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 5);
        assert_push_stack(&vec[0], nint, nint); // push right param
        assert_eq!(vec[1], Push::new(ecx).into()); // push left param
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], StackAlloc::new(-(nint * 2) as i32).into()); // caller stack cleanup
        assert_eq!(vec[4], Return::new(nint as usize).into()); // callee stack cleanup (only non-register parameter)
    }

    #[test]
    fn ms_cdecl_to_thiscall_optimized() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters(
            CallingConvention::cdecl(),
            CallingConvention::microsoft_thiscall(),
            true,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 5);
        assert_push_stack(&vec[0], nint, nint); // push right param
        assert_eq!(vec[1], Push::new(ecx).into()); // push left param
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], StackAlloc::new((-nint * 2) as i32).into()); // caller clean 2 cdecl parameters
        assert_eq!(vec[4], Return::new(4).into()); // cleanup nint on stack (thiscall right param)
    }

    #[test]
    fn ms_cdecl_to_fastcall_unoptimized() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters(
            CallingConvention::cdecl(),
            CallingConvention::fastcall(),
            false,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 5);
        assert_eq!(vec[0], Push::new(edx).into()); // push right param
        assert_eq!(vec[1], Push::new(ecx).into()); // push left param
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], StackAlloc::new(-(nint * 2) as i32).into()); // caller stack cleanup
        assert_eq!(vec[4], Return::new(0).into()); // callee stack cleanup (only non-register parameter)
    }

    #[test]
    fn ms_cdecl_to_fastcall_optimized() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters(
            CallingConvention::cdecl(),
            CallingConvention::fastcall(),
            true,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 5);
        assert_eq!(vec[0], Push::new(edx).into()); // push right param
        assert_eq!(vec[1], Push::new(ecx).into()); // push left param
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], StackAlloc::new(-8).into()); // pop right param and left param
        assert_eq!(vec[4], Return::new(0).into()); // nothing to pop
    }

    // EXTRA X86-LIKE TESTS //

    #[test]
    fn ms_stdcall_to_thiscall_optimized() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters(
            CallingConvention::stdcall(),
            CallingConvention::microsoft_thiscall(),
            true,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 4);
        assert_push_stack(&vec[0], nint, nint); // push right param
        assert_eq!(vec[1], Push::new(ecx).into()); // push left param
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], Return::new(nint as usize).into()); // callee stack cleanup (only non-register parameter)
    }

    #[test]
    fn ms_thiscall_to_stdcall_optimized() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters(
            CallingConvention::microsoft_thiscall(),
            CallingConvention::stdcall(),
            true,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 4);
        assert_push_stack(&vec[0], nint, nint); // re-push right param
        assert_eq!(vec[1], MovFromStack::new((nint * 3) as i32, ecx).into()); // mov left param to register
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], Return::new((nint * 2) as usize).into()); // caller cleanup, so no offset here
    }

    // CUSTOM FUNCTION TESTS //

    #[test]
    fn ms_cdecl_to_cdecl_with_stack_optimized() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters(&CDECL_WITH_STACK, CallingConvention::cdecl(), true);

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 6);
        assert_push_stack(&vec[0], 4, 4); // push left param
        assert_push_stack(&vec[1], 12, 4); // push right param
        assert_eq!(
            vec[2],
            StackAlloc::new(CDECL_WITH_STACK_EXTRA_SPACE as i32).into()
        );
        assert_eq!(vec[3], CallRel::new(4096).into());
        assert_eq!(
            vec[4],
            StackAlloc::new(((-nint * 2) as usize - CDECL_WITH_STACK_EXTRA_SPACE as usize) as i32)
                .into()
        ); // callee stack cleanup (2 cdecl parameters) + extra space
        assert_eq!(vec[5], Return::new(0).into());
    }

    #[test]
    fn ms_cdecl_to_cdecl_with_vector_reg_optimized() {
        let nint = size_of::<u32>() as isize;
        let result = two_parameters(&CDECL_WITH_VECTOR_REG, CallingConvention::cdecl(), true);

        assert!(result.is_ok());
        let vec: Vec<Operation<Register>> = result.unwrap();
        assert_eq!(vec.len(), 8);
        assert_eq!(vec[0], MovToStack::new(-16, xmm0).into()); // callee save xmm
        assert_eq!(vec[1], StackAlloc::new(28).into()); // 16 (xmm) - 4 (reg size)
        assert_push_stack(&vec[2], 32, 4); // push left param
        assert_push_stack(&vec[3], 40, 4); // push right param
        assert_eq!(vec[4], CallRel::new(4096).into());
        assert_eq!(vec[5], MovFromStack::new(-20, xmm0).into()); // callee restore xmm
        assert_eq!(vec[6], StackAlloc::new(-(nint as i32 * 2) - 16 - 12).into()); // caller stack cleanup (2 cdecl parameters) + 1 xmm reg + padding from xmm callee save
        assert_eq!(vec[7], Return::new(0).into());
    }

    /// Creates the instructions responsible for wrapping one object kind to another.
    ///
    /// # Parameters
    ///
    /// - `conv_called` - The calling convention to convert to `conv_current`. This is the convention of the function (`options.target_address`) called.
    /// - `conv_current` - The target convention to which convert to `conv_called`. This is the convention of the function returned.
    /// - `optimized` - Whether to generate optimized code
    fn two_parameters(
        conv_called: &CallingConvention,
        conv_current: &CallingConvention,
        optimized: bool,
    ) -> Result<Vec<Operation<Register>>, WrapperGenerationError> {
        two_parameters_with_address(conv_called, conv_current, optimized, 4096)
    }

    /// Creates the instructions responsible for wrapping one object kind to another.
    ///
    /// # Parameters
    ///
    /// - `conv_called` - The calling convention to convert to `conv_current`. This is the convention of the function (`options.target_address`) called.
    /// - `conv_current` - The target convention to which convert to `conv_called`. This is the convention of the function returned.
    /// - `optimized` - Whether to generate optimized code
    /// - `target_address` - Address to jump to.
    fn two_parameters_with_address(
        conv_called: &CallingConvention,
        conv_current: &CallingConvention,
        optimized: bool,
        target_address: usize,
    ) -> Result<Vec<Operation<Register>>, WrapperGenerationError> {
        // Specifying i32 in case test is ran from 64-bit.
        // Normally it's idiomatic to specify 'nint'.

        // Two parameters
        let mock_function = BasicFunctionInfo::new(&[ParameterType::i32, ParameterType::i32]);

        let options = get_common_options(
            optimized,
            target_address,
            target_address < 0x7FFFFFFF,
            &mock_function,
        );

        generate_wrapper_instructions::<
            Register,
            GenericCallingConvention<Register>,
            BasicFunctionInfo,
        >(conv_called, conv_current, &options)
    }

    fn get_common_options<'a>(
        optimized: bool,
        target_address: usize,
        can_generate_relative: bool,
        mock_function: &'a BasicFunctionInfo,
    ) -> WrapperInstructionGeneratorOptions<'a, BasicFunctionInfo<'a>> {
        WrapperInstructionGeneratorOptions {
            stack_entry_alignment: size_of::<u32>(), // no_alignment
            target_address,                          // some arbitrary address
            function_info: mock_function,
            injected_parameter: None,
            jit_capabilities: JitX86::get_jit_capabilities(),
            can_generate_relative_jumps: can_generate_relative,
            enable_optimizations: optimized,
            standard_register_size: size_of::<u32>(),
        }
    }

    fn assert_push_stack(op: &Operation<Register>, offset: isize, item_size: isize) {
        if let Operation::PushStack(x) = op {
            assert!(x.has_offset_and_size(offset as i32, item_size as u32));
        }
    }

    const CDECL_WITH_STACK_EXTRA_SPACE: u32 = 32;
    static CDECL_WITH_STACK: CallingConvention = CallingConvention {
        convention: GenericCallingConvention::<Register> {
            int_parameters: &[],
            float_parameters: &[],
            vector_parameters: &[],
            return_register: eax,
            reserved_stack_space: CDECL_WITH_STACK_EXTRA_SPACE,
            callee_saved_registers: &[ebx, esi, edi, ebp],
            always_saved_registers: &[],
            stack_cleanup: StackCleanup::Caller,
            stack_parameter_order: StackParameterOrder::RightToLeft,
            required_stack_alignment: 1,
        },
    };

    static CDECL_WITH_VECTOR_REG: CallingConvention = CallingConvention {
        convention: GenericCallingConvention::<Register> {
            int_parameters: &[],
            float_parameters: &[],
            vector_parameters: &[],
            return_register: eax,
            reserved_stack_space: 0,
            callee_saved_registers: &[ebx, esi, edi, ebp, xmm0],
            always_saved_registers: &[],
            stack_cleanup: StackCleanup::Caller,
            stack_parameter_order: StackParameterOrder::RightToLeft,
            required_stack_alignment: 1,
        },
    };
}
