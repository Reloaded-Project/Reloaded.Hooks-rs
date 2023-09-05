use derive_new::new;

/// Represents a push stack operation which pushes a value onto the stack from an
/// offset relative to the current stack pointer.
///
/// # Fields
///
/// - `offset`: This is the offset from the base register.  
/// - `item_size`: Size of the item to push.  
///
/// # Example
///
/// The `PushStackOperation` can be used to represent operations where a value is taken from
/// an address relative to a register and then pushed onto the stack.
///
/// For instance, in x86 assembly, the instruction `push [esp+8]` would push the value
/// located at an offset of 8 bytes from the `esp` register onto the stack. This can be
/// modeled using `PushStackOperation` as follows:
///
/// ```
/// use reloaded_hooks_portable::api::jit::push_stack_operation::PushStackOperation;
/// let push_offset_from_esp = PushStackOperation { offset: 8, item_size: 4 };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, new)]
pub struct PushStackOperation {
    /// The offset from the current stack pointer in the direction opposite to the stack's growth.
    pub offset: isize,

    /// Size of the item to re-push to stack.
    pub item_size: usize,
}
