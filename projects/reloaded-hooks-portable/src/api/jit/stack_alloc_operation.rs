use derive_new::new;

/// Represents a stack allocation operation which adds or subtracts the current stack pointer.
///
/// This is usually represented as something like `sub esp, 4` or `add sp, 4`.
/// The direction of stack growth can vary depending on architecture; so implementations
/// of this should use the default for the architecture.
///
/// In the case of x86, this is subtracting from the stack pointer, so the stack grows
/// downwards; but on some ARM platforms etc. this means adding to the stack pointer instead.
///
/// # Fields
///
/// `operand`: This represents the amount of bytes the stack should be adjusted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, new)]
pub struct StackAllocOperation {
    /// The amount of space to reserve on the stack.
    /// If this value is negative, the stack shrinks.
    pub operand: i32,
}
