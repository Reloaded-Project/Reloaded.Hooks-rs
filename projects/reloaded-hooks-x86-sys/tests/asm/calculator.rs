#[cfg(target_arch = "x86_64")]
pub type Add = extern "win64" fn(i64, i64) -> i64;

// add_msft_x64.asm
#[cfg(target_arch = "x86_64")]
pub const CALCULATOR_ADD_MSFT_X64: [u8; 14] = [
    0x48, 0x89, 0xC8, 0x48, 0x01, 0xD0, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0xC3,
];

#[cfg(target_arch = "x86")]
pub type Add = extern "cdecl" fn(i32, i32) -> i32;

// add_x86.asm
#[cfg(target_arch = "x86")]
pub const CALCULATOR_ADD_CDECL_X86: [u8; 11] = [
    0x55, 0x89, 0xE5, 0x8B, 0x45, 0x08, 0x03, 0x45, 0x0C, 0x5D, 0xC3,
];
