/// Implemented by register types which need to express their size in bytes to the world.
pub trait RegisterInfo {
    /// Returns the size of the register in bytes.
    fn size_in_bytes(&self) -> usize;

    /// True if the register is a stack pointer.
    fn is_stack_pointer(&self) -> bool;

    /// Returns the 'type' of register this individual register represents.  
    ///
    /// The wrapper generator optimizer will prevent registers from different 'types'
    /// to participate in the same optimizations.  
    ///
    /// Usually you want to have a type for `float` registers and a type for `int` registers.
    ///
    /// # Explanation
    ///
    /// Architectures will often have registers that are not compatible with each other, such
    /// as floating point registers and integer registers.
    ///
    /// For example, consider the sequence where we want to mov a double
    /// from a floating point register to a general purpose register under ARM64:
    ///
    /// ```asm
    /// ; This is 'PushOperation' in the reloaded-hooks-portable
    /// sub sp, sp, #8     ; Allocate 8 bytes on the stack for a 64-bit value
    /// str d0, [sp]      ; Store d0 onto the stack
    ///
    /// ; This is 'PopOperation' in the reloaded-hooks-portable
    /// ldr x1, [sp]      ; Load the value from the stack into x1
    /// add sp, sp, #8    ; Adjust the stack pointer back
    /// ```
    ///
    /// The wrapper generator might optimize this as the following sequence:
    ///
    /// ```
    /// use reloaded_hooks_portable::api::jit::mov_operation::MovOperation;
    ///
    /// let move_op = MovOperation {
    ///     source: "d0",
    ///     target: "x1"
    /// };
    /// ```
    ///
    /// While this is valid code for the JIT, ARM64 is not capable of this, as data
    /// cannot be transferred directly between a floating point register and a general
    /// purpose register.
    ///
    /// To prevent this from happening, you set a different register type for floating
    /// point registers and general purpose registers, so the optimizer will not
    /// attempt to optimize them together.
    fn register_type(&self) -> usize;
}
