#[macro_export]
macro_rules! mov_item_to_stack {
    ($a:expr, $reg:expr, $offset:expr, $convert_method:ident, $op:ident) => {
        use alloc::string::ToString;
        match $a.bitness() {
            #[cfg(feature = "x86")]
            32 => {
                $a.$op(
                    iced_x86::code_asm::dword_ptr(iced_x86::code_asm::registers::esp) + $offset,
                    $reg.$convert_method()?,
                )
                .map_err($crate::common::jit_common::convert_error)?;
            }
            #[cfg(feature = "x64")]
            64 => {
                $a.$op(
                    iced_x86::code_asm::qword_ptr(iced_x86::code_asm::registers::rsp) + $offset,
                    $reg.$convert_method()?,
                )
                .map_err($crate::common::jit_common::convert_error)?;
            }
            _ => {
                return Err(JitError::ThirdPartyAssemblerError(
                    $crate::common::jit_common::ARCH_NOT_SUPPORTED.to_string(),
                ));
            }
        }
    };
}
