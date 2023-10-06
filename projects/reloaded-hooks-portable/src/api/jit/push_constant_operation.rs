use derive_new::new;

/// Represents an operation which pushes a constant `usize` onto the stack.
///
/// This can be used to model assembly operations that push a constant value onto the stack.
/// For instance, in x86 assembly, this could be modeled by a `push` instruction with an immediate
/// value.
///
/// # Example
///
/// The `PushConstantOperation` can be used to represent assembly operations that push a constant
/// value onto the stack. For instance, in x86 assembly, the instruction `push 10` would push the
/// constant value `10` onto the stack. This can be modeled using `PushConstantOperation`
/// as follows:
///
/// ```
/// use reloaded_hooks_portable::api::jit::push_constant_operation::PushConstantOperation;
/// let push_10 = PushConstantOperation::<i32> { value: 10, scratch: None };
/// ```
///
/// Similarly, in architectures without an explicit `push` instruction for constants, this operation
/// can be modeled using other appropriate instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, new)]
pub struct PushConstantOperation<T> {
    /// The constant value to push onto the stack.
    pub value: usize,

    /// Scratch register to use for the push operation. (Needed for some architectures)
    pub scratch: Option<T>,
}
