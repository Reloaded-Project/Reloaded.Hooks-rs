extern crate alloc;

/*
use crate::{
    api::{
        buffers::buffer_abstractions::{Buffer, BufferFactory},
        jit::compiler::Jit,
        length_disassembler::LengthDisassembler,
        rewriter::code_rewriter::CodeRewriter,
        traits::register_info::RegisterInfo,
    },
    helpers::make_inline_rel_branch::INLINE_BRANCH_LEN,
};

use bitfield::bitfield;
use core::marker::PhantomData;

/// Represents an assembly hook.
#[repr(C)] // Not 'packed' because this is not in array and malloc in practice will align this.
pub struct BranchHook<TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>
where
    TBuffer: Buffer,
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBufferFactory: BufferFactory<TBuffer>,
{
    // docs/dev/design/branch-hooks/overview.md
    /// The address of the stub containing custom code.
    stub_address: usize, // 0

    /// Stores sizes of 'branch_to_orig_opcode' and 'branch_to_hook_opcode'.
    props: BranchHookPackedProps, // 4/8

    /// The code to branch to 'orig' segment in the buffer, when disabling the hook.
    branch_to_orig_opcode: [u8; INLINE_BRANCH_LEN], // 5/9

    /// The code to restore previous functionality from before `branch_to_orig_opcode` was written.
    restore_hook_opcode: [u8; INLINE_BRANCH_LEN], // 10/14

    // ~1 byte left
    // 15/19 (x86/x64)
    // 17 (AArch64)

    // Dummy type parameters for Rust compiler to comply.
    _unused_buf: PhantomData<TBuffer>,
    _unused_tj: PhantomData<TJit>,
    _unused_tr: PhantomData<TRegister>,
    _unused_td: PhantomData<TDisassembler>,
    _unused_rwr: PhantomData<TRewriter>,
    _unused_fac: PhantomData<TBufferFactory>,
}

impl<TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>
    BranchHook<TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>
where
    TBuffer: Buffer,
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBufferFactory: BufferFactory<TBuffer>,
{
    /// Writes the hook to memory, either enabling or disabling it based on the provided parameters.
    fn write_hook(&self, branch_opcode: &[u8], code: &[u8], num_bytes: usize) {
        // Write the branch first, as per docs
        // This also overwrites some extra code afterwards, but that's a-ok for now.
        unsafe {
            atomic_write_masked::<TBuffer>(self.stub_address, branch_opcode, num_bytes);
        }

        // Now write the remaining code
        TBuffer::overwrite(self.stub_address + num_bytes, &code[num_bytes..]);

        // And now re-insert the code we temp overwrote with the branch
        unsafe {
            atomic_write_masked::<TBuffer>(self.stub_address, code, num_bytes);
        }
    }

    /// Enables the hook.
    /// This will cause the hook to be written to memory.
    /// If the hook is already enabled, this function does nothing.
    /// If the hook is disabled, this function will write the hook to memory.
    pub fn enable(&self) {
        unsafe {
            let props = self.props.as_ref();
            let num_bytes = props.get_branch_to_hook_len();
            self.write_hook(
                props.get_branch_to_hook_slice(),
                props.get_enabled_code(),
                num_bytes,
            );
        }
    }

    /// Disables the hook.
    /// This will cause the hook to be no-opped.
    /// If the hook is already disabled, this function does nothing.
    /// If the hook is enabled, this function will no-op the hook.
    pub fn disable(&self) {
        unsafe {
            let props = self.props.as_ref();
            let num_bytes = props.get_branch_to_orig_len();
            self.write_hook(
                props.get_branch_to_orig_slice(),
                props.get_disabled_code(),
                num_bytes,
            );
        }
    }

    /// Returns true if the hook is enabled, else false.
    pub fn get_is_enabled(&self) -> bool {
        unsafe { self.props.as_ref().is_enabled() }
    }
}
*/
