#[cfg(target_arch = "x86_64")]
pub type Add = extern "win64" fn(i64, i64) -> i64;

// add_msft_x64.asm
pub const CALCULATOR_ADD_MSFT_X64: [u8; 14] = [
    0x48, 0x89, 0xC8, 0x48, 0x01, 0xD0, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0xC3,
];

// call_add_msft_x64.asm
pub const CALL_CALCULATOR_ADD_MSFT_X64: [u8; 31] = [
    // add_fn:
    0x48, 0x89, 0xC8, // mov rax, rcx
    0x48, 0x01, 0xD0, // add rax, rdx
    0xC3, // ret
    // add_wrapper:
    0x48, 0x83, 0xEC, 0x40, // sub rsp, 40h
    0xE8, 0xF0, 0xFF, 0xFF, 0xFF, // call add_fn (Placeholder offset)
    0x48, 0x83, 0xC4, 0x40, // add rsp, 40h
    0xC3, // ret
    // target_function:
    0x48, 0x89, 0xC8, // mov rax, rcx
    0x48, 0x01, 0xD0, // add rax, rdx
    0x48, 0xFF, 0xC0, // inc rax
    0xC3, // ret
];

// Update the offsets according to the new array layout
pub const CALL_CALCULATOR_ADD_MSFT_X64_FUN_OFFSET: usize = 7; // Start of add_wrapper
pub const CALL_CALCULATOR_ADD_MSFT_X64_CALL_OFFSET: usize = 11; // Offset of 'call add_fn' in add_wrapper
pub const CALL_CALCULATOR_ADD_MSFT_X64_TARGET_FUNCTION_OFFSET: usize = 21; // Start of target_function

//////////////////// X86 ////////////////////

#[cfg(target_arch = "x86")]
pub type Add = extern "cdecl" fn(i32, i32) -> i32;

// add_x86.asm
pub const CALCULATOR_ADD_CDECL_X86: [u8; 11] = [
    0x55, 0x89, 0xE5, 0x8B, 0x45, 0x08, 0x03, 0x45, 0x0C, 0x5D, 0xC3,
];

// call_add_x86.asm
pub const CALL_CALCULATOR_ADD_CDECL_X86: [u8; 40] = [
    0x55, 0x89, 0xE5, 0x8B, 0x45, 0x08, 0x03, 0x45, 0x0C, 0x5D, 0xC3, 0xFF, 0x74, 0x24, 0x08, 0xFF,
    0x74, 0x24, 0x08, 0xE8, 0xE8, 0xFF, 0xFF, 0xFF, 0x83, 0xC4, 0x08, 0xC3, 0x55, 0x89, 0xE5, 0x8B,
    0x45, 0x08, 0x03, 0x45, 0x0C, 0x40, 0x5D, 0xC3,
];

// Offset of the start of function to run in  CALL_CALCULATOR_ADD_CDECL_X86.
pub const CALL_CALCULATOR_ADD_CDECL_X86_FUN_OFFSET: usize = 11;

// Offset of the call instruction in CALL_CALCULATOR_ADD_CDECL_X86 to be hooked.
pub const CALL_CALCULATOR_ADD_CDECL_X86_CALL_OFFSET: usize = 19;

// Offset of the target function in CALL_CALCULATOR_ADD_MSFT_X64 to replace the original branch with.
// Used for testing .
pub const CALL_CALCULATOR_ADD_CDECL_X86_TARGET_FUNCTION_OFFSET: usize = 28;
