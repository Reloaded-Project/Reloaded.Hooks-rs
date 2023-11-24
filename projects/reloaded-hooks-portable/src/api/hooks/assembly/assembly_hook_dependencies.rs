use crate::api::{
    jit::compiler::Jit, length_disassembler::LengthDisassembler,
    rewriter::code_rewriter::CodeRewriter, traits::register_info::RegisterInfo,
};
use core::marker::PhantomData;

/// Represents the input dependencies required to create an assembly hook.
/// These are the requirements on the architecture side (i.e. x86 specific code)
pub struct AssemblyHookDependencies<'a, TJit, TRegister, TDisassembler, TRewriter>
where
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
{
    pub(crate) jit: &'a TJit,

    _phantom_dis: PhantomData<TDisassembler>,
    _phantom_reg: PhantomData<TRegister>,
    _phantom_rw: PhantomData<TRewriter>,
}

impl<'a, TJit, TRegister, TDisassembler, TRewriter>
    AssemblyHookDependencies<'a, TJit, TRegister, TDisassembler, TRewriter>
where
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
{
    /// Creates a new instance of `AssemblyHookDependencies`.
    ///
    /// # Arguments
    /// - `jit`: A reference to the JIT instance, e.g. JitX64.
    pub fn new(jit: &'a TJit) -> Self {
        Self {
            jit,
            _phantom_reg: PhantomData,
            _phantom_dis: PhantomData,
            _phantom_rw: PhantomData,
        }
    }
}
