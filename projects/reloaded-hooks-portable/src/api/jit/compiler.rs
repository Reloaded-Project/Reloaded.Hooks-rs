extern crate alloc;

use super::operation::Operation;
use crate::api::traits::register_info::RegisterInfo;
use alloc::{rc::Rc, string::String};
use core::fmt::Debug;
use thiserror_no_std::Error;

/// Lists the supported features of the JIT
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JitCapabilities {
    /// Can encode call that is relative to the instruction pointer.
    /// This controls whether [CallIpRelativeOperation](super::call_rip_relative_operation::CallIpRelativeOperation) is emitted.
    CanEncodeIPRelativeCall,

    /// Can encode jump that is relative to the instruction pointer.
    /// This controls whether [JumpIpRelativeOperation](super::jump_rip_relative_operation::JumpIpRelativeOperation) is emitted.
    CanEncodeIPRelativeJump,

    /// Can encode multiple push/pop operations at once.
    /// This controls whetehr [MultiPush](super::push_operation::PushOperation) and [MultiPop](super::pop_operation::PopOperation) are emitted.
    CanMultiPush,
}

/// The trait for a Just In Time Compiler used for emitting
/// wrappers assembled for a given address.
pub trait Jit<TRegister: RegisterInfo> {
    /// Compiles the specified sequence of operations into a sequence of bytes.
    fn compile(
        &mut self,
        address: usize,
        operations: &[Operation<TRegister>],
    ) -> Result<Rc<[u8]>, JitError<TRegister>>;

    /// Required alignment of code for the current architecture.
    ///
    /// # Remarks
    /// This is usually 4 bytes on most architectures, and 16 bytes on x86.
    fn code_alignment() -> u32;

    /// Maximum distance of relative jump assembly instruction.
    /// This affects wrapper generation, and parameters passed into JIT.
    fn max_relative_jump_distance() -> usize;

    /// Returns the functionalities supported by this JIT.
    /// These functionalities affect code generation performed by this library.
    fn get_jit_capabilities() -> &'static [JitCapabilities];
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

    #[error("Operand is out of range: {0:?}")]
    OperandOutOfRange(String),
}

pub fn transform_err<TOldRegister: Clone + Copy, TNewRegister, TConvertRegister>(
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
        JitError::OperandOutOfRange(a) => JitError::OperandOutOfRange(a),
    }
}
