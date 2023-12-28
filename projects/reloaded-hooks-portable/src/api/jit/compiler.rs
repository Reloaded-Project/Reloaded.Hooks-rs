extern crate alloc;

use super::{
    call_relative_operation::CallRelativeOperation, jump_absolute_operation::JumpAbsoluteOperation,
    jump_relative_operation::JumpRelativeOperation, operation::Operation,
};
use crate::api::traits::register_info::RegisterInfo;
use alloc::{string::String, vec::Vec};
use bitflags::bitflags;
use core::fmt::Debug;
use derive_new::new;
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

        /// This JIT can perform a 'relative jump' operation to any address.
        const CAN_RELATIVE_JUMP_TO_ANY_ADDRESS = 1 << 4;

        /// This JIT can perform the 'Mov To Stack' operation.
        const CAN_MOV_TO_STACK = 1 << 5;
    }
}

/// The trait for a Just In Time Compiler used for emitting
/// wrappers assembled for a given address.
pub trait Jit<TRegister: RegisterInfo> {
    /// Compiles the specified sequence of operations into a sequence of bytes.
    fn compile(
        address: usize,
        operations: &[Operation<TRegister>],
    ) -> Result<Vec<u8>, JitError<TRegister>>;

    /// Compiles the specified sequence of operations into a sequence of bytes.
    fn compile_with_buf(
        address: usize,
        operations: &[Operation<TRegister>],
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<TRegister>>;

    /// Required alignment of code for the current architecture.
    ///
    /// # Remarks
    /// This is usually 4 bytes on most architectures, and 16 bytes on x86.
    fn code_alignment() -> u32;

    /// Maximum number of bytes required to perform a branch (i.e. an absolute branch).
    fn max_branch_bytes() -> u32;

    /// Stack offset upon entry into a method from desired value.
    ///
    /// This is 0 for architectures with a link register, or sizeof([usize]) for architectures which have
    /// return addresses on stack.
    fn stack_entry_misalignment() -> u32;

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

    /// Returns the size of a regular register in bytes.
    fn standard_register_size() -> usize;

    // TODO: Consider moving these things to 'JITUtils' or something.

    /// Maximum distance of 'call' or 'branch and link' operation.
    fn max_standard_relative_call_distance() -> usize;

    /// Number of bytes used to encode a 'standard' relative call instruction. [`Self::max_standard_relative_call_distance`]
    fn standard_relative_call_bytes() -> usize;

    /// Fills an array with NOP instructions.
    fn fill_nops(arr: &mut [u8]);

    /// Assembles a 'jmp'/'branch' instruction directly, bypassing the whole compilation step.
    /// This is used to speed up single instruction computation.
    ///
    /// # Parameters
    /// - `x` - The jump instruction to encode.
    /// - `pc` - The current program counter.
    /// - `buf` - The buffer to write the instruction to.
    fn encode_jump(
        x: &JumpRelativeOperation<TRegister>,
        pc: &mut usize,
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<TRegister>>;

    /// Assembles a 'jmp'/'branch' instruction to an absolute address, bypassing
    /// the whole compilation step. This is used to speed up single instruction computation.
    ///
    /// # Parameters
    /// - `x` - The jump instruction to encode.
    /// - `pc` - The current program counter.
    /// - `buf` - The buffer to write the instruction to.
    fn encode_abs_jump(
        x: &JumpAbsoluteOperation<TRegister>,
        pc: &mut usize,
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<TRegister>>;

    /// Assembles a 'call'/'branch link' instruction directly, bypassing the whole compilation step.
    /// This is used to speed up single instruction computation.
    ///
    /// # Parameters
    /// - `x` - The call instruction to encode.
    /// - `pc` - The current program counter.
    /// - `buf` - The buffer to write the instruction to.
    fn encode_call(
        x: &CallRelativeOperation,
        pc: &mut usize,
        buf: &mut Vec<u8>,
    ) -> Result<(), JitError<TRegister>>;

    /// Decodes the target address of a 'call instruction', i.e. the address where the 'call'
    /// instruction will branch to.
    ///
    /// # Parameters
    /// - `ins_address` - The address of the 'call' instruction.
    /// - `ins_length` - The length of the 'call' instruction at 'ins_address'.
    ///
    /// # Returns
    ///
    /// Returns the target address of the call instruction.
    /// Otherwise an error.
    fn decode_call_target(
        ins_address: usize,
        ins_length: usize,
    ) -> Result<DecodeCallTargetResult, &'static str>;

    /// Maximum number of bytes required to perform a relative jump.
    /// This is the max amount of bytes that can be returned by [`self::encode_jump`].
    fn max_relative_jump_bytes() -> usize;
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

/// Contains the result of decoding a 'call' instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, new)]
pub struct DecodeCallTargetResult {
    /// The target address of the call instruction.
    pub target_address: usize,

    /// True if the instruction is a 'call' (branch+link) instruction.
    /// False if the instruction is a 'jump' (branch) instruction.
    pub is_call: bool,
}
