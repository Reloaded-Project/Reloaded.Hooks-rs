extern crate alloc;
use alloc::string::ToString;
use iced_x86::IcedError;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use zydis::Status;

#[derive(Debug)]
pub enum X86jitError<T> {
    Iced(IcedError),
    Zydis(Status),
    Jit(JitError<T>),
}

impl<T> From<IcedError> for X86jitError<T> {
    fn from(e: IcedError) -> Self {
        Self::Iced(e)
    }
}

impl<T> From<Status> for X86jitError<T> {
    fn from(e: Status) -> Self {
        Self::Zydis(e)
    }
}

impl<T> From<JitError<T>> for X86jitError<T> {
    fn from(e: JitError<T>) -> Self {
        Self::Jit(e)
    }
}

impl<T> From<X86jitError<T>> for JitError<T> {
    fn from(val: X86jitError<T>) -> Self {
        match val {
            X86jitError::Jit(e) => e,
            X86jitError::Iced(e) => JitError::ThirdPartyAssemblerError(e.to_string()),
            X86jitError::Zydis(e) => JitError::ThirdPartyAssemblerError(e.to_string()),
        }
    }
}
