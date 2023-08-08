extern crate alloc;

use alloc::{rc::Rc, string::String};
use core::fmt::Debug;

use thiserror_no_std::Error;

use super::operation::Operation;

/// The trait for a Just In Time Compiler used for emitting
/// wrappers assembled for a given address.
pub trait Jit<TRegister> {
    /// Compiles the specified sequence of operations into a sequence of bytes.
    fn compile(
        &mut self,
        address: usize,
        operations: &[Operation<TRegister>],
    ) -> Result<Rc<[u8]>, JitError<TRegister>>;
}

/// Errors that can occur during JIT compilation.
#[derive(Debug, Error)]
pub enum JitError<TRegister> {
    /// Failed to initialize 3rd party assembler
    #[error("Cannot initialize Assembler: {0:?}")]
    CannotInitializeAssembler(String),

    /// Error related to 3rd party assembler.
    #[error("3rd Party Assembler Error: {0:?}")]
    ThirdPartyAssemblerError(String),

    // Invalid Register Used
    #[error("Invalid Register Used: {0:?}")]
    InvalidRegister(TRegister),

    /// Invalid register pair
    #[error("The two given registers cannot be used together for this opcode: {0:?} {1:?}")]
    InvalidRegisterCombination(TRegister, TRegister),

    /// JIT of an unrecognised instruction was requested.
    #[error("Invalid instruction provided: {0:?}")]
    InvalidInstruction(Operation<TRegister>),
}

pub fn transform_err<TOldRegister, TNewRegister, TConvertRegister>(
    err: JitError<TOldRegister>,
    f: TConvertRegister,
) -> JitError<TNewRegister>
where
    TConvertRegister: Fn(TOldRegister) -> TNewRegister,
{
    match err {
        JitError::CannotInitializeAssembler(x) => JitError::CannotInitializeAssembler(x),
        JitError::ThirdPartyAssemblerError(x) => JitError::ThirdPartyAssemblerError(x),
        JitError::InvalidRegister(x) => JitError::InvalidRegister(f(x)),
        JitError::InvalidInstruction(x) => {
            JitError::InvalidInstruction(super::operation::transform_op(x, f))
        }
        JitError::InvalidRegisterCombination(a, b) => {
            JitError::InvalidRegisterCombination(f(a), f(b))
        }
    }
}
