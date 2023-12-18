extern crate alloc;

use super::operation::Operation;
use crate::api::traits::register_info::RegisterInfo;
use alloc::{rc::Rc, string::String};
use bitflags::bitflags;
use core::fmt::Debug;
use thiserror_no_std::Error;

bitflags! {
    /// Lists the supported features of the JIT.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct JitCapabilities: u32 {
        /// Can encode call that is relative to the instruction pointer.
        /// This controls whether `CallIpRelativeOperation` (super::call_rip_relative_operation::CallIpRelativeOperation) is emitted.
        const CAN_ENCODE_IP_RELATIVE_CALL = 1 << 0;

        /// Can encode jump that is relative to the instruction pointer.
        /// This controls whether `JumpIpRelativeOperation` (super::jump_rip_relative_operation::JumpIpRelativeOperation) is emitted.
        const CAN_ENCODE_IP_RELATIVE_JUMP = 1 << 1;

        /// Can encode multiple push/pop operations at once.
        /// This controls whether `MultiPush` (super::push_operation::PushOperation) and
        /// `MultiPop` (super::pop_operation::PopOperation) are emitted.
        const CAN_MULTI_PUSH = 1 << 2;

        /// Set this flag if Jit can produce an 'Absolute Indirect Jump' that uses less bytes
        /// than an 'Absolute Jump'. This is used for Assembly Hooks to reduce number of bytes
        /// used.
        const PROFITABLE_ABSOLUTE_INDIRECT_JUMP = 1 << 3;
    }
}

/// The trait for a Just In Time Compiler used for emitting
/// wrappers assembled for a given address.
pub trait Jit<TRegister: RegisterInfo> {
    /// Compiles the specified sequence of operations into a sequence of bytes.
    fn compile(
        address: usize,
        operations: &[Operation<TRegister>],
    ) -> Result<Rc<[u8]>, JitError<TRegister>>;

    /// Required alignment of code for the current architecture.
    ///
    /// # Remarks
    /// This is usually 4 bytes on most architectures, and 16 bytes on x86.
    fn code_alignment() -> u32;

    /// Maximum number of bytes required to perform a branch (i.e. an absolute branch).
    fn max_branch_bytes() -> u32;

    /// Maximum distances of supported relative jump assembly instruction sequences.
    /// This affects wrapper generation, and parameters passed into JIT.
    fn max_relative_jump_distances() -> &'static [usize];

    /// Returns the functionalities supported by this JIT.
    /// These functionalities affect code generation performed by this library.
    fn get_jit_capabilities() -> JitCapabilities;

    /// Max Offset used for Absolute Indirect Jump.
    /// a.k.a. [`reloaded_hooks_portable::api::jit::jump_absolute_indirect_operation::JumpAbsoluteIndirectOperation`]
    ///
    /// Override this if you set [`reloaded_hooks_portable::api::jit::compiler::JitCapabilities::PROFITABLE_ABSOLUTE_INDIRECT_JUMP`] in [`self::get_jit_capabilities`]
    fn max_indirect_offsets() -> &'static [u32] {
        &[]
    }

    /// Fills an array with NOP instructions.
    fn fill_nops(arr: &mut [u8]);
}

/// Errors that can occur during JIT compilation.
#[derive(Debug, Error)]
pub enum JitError<TRegister> {
    /// Failed to initialize 3rd party assembler
    #[error("Cannot initialize Assembler: {0:?}")]
    CannotInitializeAssembler(String),

    /// No scratch register found.
    #[error("No Scratch Register Found: {0:?}")]
    NoScratchRegister(String),

    /// Error related to 3rd party assembler.
    #[error("3rd Party Assembler Error: {0:?}")]
    ThirdPartyAssemblerError(String),

    // Invalid Register Used
    #[error("Invalid Register Used: {0:?}")]
    InvalidRegister(TRegister),

    /// Invalid register pair
    #[error("The two given registers cannot be used together for this opcode: {0:?} {1:?}")]
    InvalidRegisterCombination(TRegister, TRegister),

    /// Invalid register triplet
    #[error(
        "The three given registers cannot be used together for this opcode: {0:?} {1:?} {2:?}"
    )]
    InvalidRegisterCombination3(TRegister, TRegister, TRegister),

    /// JIT of an unrecognised instruction was requested.
    #[error("Invalid instruction provided: {0:?}")]
    InvalidInstruction(Operation<TRegister>),

    #[error("Operand is out of range: {0:?}")]
    OperandOutOfRange(String),

    #[error("Invalid offset specified: {0:?}")]
    InvalidOffset(String),
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
        JitError::InvalidOffset(x) => JitError::InvalidOffset(x),
        JitError::NoScratchRegister(x) => JitError::NoScratchRegister(x),
        JitError::InvalidRegisterCombination3(a, b, c) => {
            JitError::InvalidRegisterCombination3(f(a), f(b), f(c))
        }
    }
}
