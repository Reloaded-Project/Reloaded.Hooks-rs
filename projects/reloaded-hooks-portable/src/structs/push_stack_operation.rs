/// Represents a push stack operation which pushes a value onto the stack from an address
/// relative to a given register (often the stack pointer).
///
/// # Fields
///
/// - `base_register`: This is the base register from which the offset is calculated.  
/// - `offset`: This is the offset from the base register.  
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
/// use reloaded_hooks_portable::structs::push_stack_operation::PushStackOperation;
/// let push_offset_from_esp = PushStackOperation { base_register: "esp", offset: 8 };
/// ```
///
/// # Note
///
/// The `PushStackOperation` only represents the operation itself; it does not perform
/// the operation or modify any actual register or memory values. To simulate the
/// effect of the operation, you would need to implement additional logic or use
/// a computing architecture simulation framework.
///
/// In the real world, you should use enums or more specific types instead of strings for `base_register`,
/// the code above shows strings for clarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PushStackOperation<T> {
    /// The base register from which the offset is calculated.
    pub base_register: T,

    /// The offset from the base register.
    pub offset: isize, // using isize to allow for both positive and negative offsets
}
