use core::mem::size_of;

use criterion::Criterion;
use lazy_static::lazy_static;
use reloaded_hooks_portable::{
    api::function_info::ParameterType,
    api::{
        errors::wrapper_generation_error::WrapperGenerationError,
        jit::{compiler::JitCapabilities, operation::Operation},
        wrapper_instruction_generator::{
            generate_wrapper_instructions, WrapperInstructionGeneratorOptions,
        },
    },
    helpers::test_helpers::MockFunction,
    helpers::test_helpers::*,
};

pub fn benchmark_wrappers(c: &mut Criterion) {
    c.bench_function("ms_thiscall_to_cdecl_unoptimized", |b| {
        b.iter(ms_thiscall_to_cdecl_unoptimized)
    });

    c.bench_function("ms_thiscall_to_cdecl_optimized", |b| {
        b.iter(ms_thiscall_to_cdecl_optimized)
    });

    c.bench_function("ms_cdecl_to_thiscall_unoptimized", |b| {
        b.iter(ms_cdecl_to_thiscall_unoptimized)
    });

    c.bench_function("ms_cdecl_to_thiscall_optimized", |b| {
        b.iter(ms_cdecl_to_thiscall_optimized)
    });

    c.bench_function("ms_cdecl_to_fastcall_unoptimized", |b| {
        b.iter(ms_cdecl_to_fastcall_unoptimized)
    });

    c.bench_function("ms_cdecl_to_fastcall_optimized", |b| {
        b.iter(ms_cdecl_to_fastcall_optimized)
    });

    c.bench_function("ms_stdcall_to_thiscall_optimized", |b| {
        b.iter(ms_stdcall_to_thiscall_optimized)
    });

    c.bench_function("ms_thiscall_to_stdcall_optimized", |b| {
        b.iter(ms_thiscall_to_stdcall_optimized)
    });
}

// X86-LIKE TESTS //

fn ms_thiscall_to_cdecl_unoptimized() -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError>
{
    two_parameters(
        &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
        &CDECL_LIKE_FUNCTION_ATTRIBUTE,
        false,
    )
}

fn ms_thiscall_to_cdecl_optimized() -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError>
{
    two_parameters(
        &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
        &CDECL_LIKE_FUNCTION_ATTRIBUTE,
        true,
    )
}

fn ms_cdecl_to_thiscall_unoptimized() -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError>
{
    two_parameters(
        &CDECL_LIKE_FUNCTION_ATTRIBUTE,
        &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
        false,
    )
}

fn ms_cdecl_to_thiscall_optimized() -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError>
{
    two_parameters(
        &CDECL_LIKE_FUNCTION_ATTRIBUTE,
        &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
        true,
    )
}

fn ms_cdecl_to_fastcall_unoptimized() -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError>
{
    two_parameters(
        &CDECL_LIKE_FUNCTION_ATTRIBUTE,
        &FASTCALL_LIKE_FUNCTION_ATTRIBUTE,
        false,
    )
}

fn ms_cdecl_to_fastcall_optimized() -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError>
{
    two_parameters(
        &CDECL_LIKE_FUNCTION_ATTRIBUTE,
        &FASTCALL_LIKE_FUNCTION_ATTRIBUTE,
        true,
    )
}

// EXTRA X86-LIKE TESTS //

fn ms_stdcall_to_thiscall_optimized() -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError>
{
    two_parameters(
        &STDCALL_LIKE_FUNCTION_ATTRIBUTE,
        &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
        true,
    )
}

fn ms_thiscall_to_stdcall_optimized() -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError>
{
    two_parameters(
        &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
        &STDCALL_LIKE_FUNCTION_ATTRIBUTE,
        true,
    )
}

lazy_static! {
    static ref MOCK_FUNCTION: MockFunction = MockFunction {
        parameters: vec![ParameterType::nint, ParameterType::nint],
    };
}

/// Creates the instructions responsible for wrapping one object kind to another.
///
/// # Parameters
///
/// - `from_convention` - The calling convention to convert to `to_convention`. This is the convention of the function (`options.target_address`) called.
/// - `to_convention` - The target convention to which convert to `from_convention`. This is the convention of the function returned.
/// - `optimized` - Whether to generate optimized code
fn two_parameters(
    from_convention: &MockFunctionAttribute,
    to_convention: &MockFunctionAttribute,
    optimized: bool,
) -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError> {
    // Two parameters
    let capabiltiies = JitCapabilities::CAN_ENCODE_IP_RELATIVE_CALL
        | JitCapabilities::CAN_ENCODE_IP_RELATIVE_JUMP
        | JitCapabilities::CAN_MULTI_PUSH;

    let options = get_common_options(optimized, &MOCK_FUNCTION, capabiltiies);
    generate_wrapper_instructions(from_convention, to_convention, options)
}

fn get_common_options(
    optimized: bool,
    mock_function: &MockFunction,
    capabilties: JitCapabilities,
) -> WrapperInstructionGeneratorOptions<MockFunction> {
    WrapperInstructionGeneratorOptions {
        stack_entry_alignment: size_of::<isize>(), // no_alignment
        target_address: 0x1000,                    // some arbitrary address
        function_info: mock_function,
        injected_parameter: None, // some arbitrary value
        jit_capabilities: capabilties,
        can_generate_relative_jumps: true,
        enable_optimizations: optimized,
    }
}
