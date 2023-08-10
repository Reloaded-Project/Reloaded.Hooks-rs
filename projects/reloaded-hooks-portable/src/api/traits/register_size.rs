/// Implemented by register types which need to express their size in bytes to the world.
pub trait RegisterInfo {
    /// Returns the size of the register in bytes.
    fn size_in_bytes(&self) -> usize;

    /// True if the register is a stack pointer.
    fn is_stack_pointer(&self) -> bool;
}
