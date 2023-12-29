use derive_new::new;

/// Represents a push operation which pushes a register onto the stack and increments
/// the current stack pointer.
///
/// This can be used to model assembly `push` instructions.
/// (Or in cases of architectures without an explicit `push` instruction, `sub` stack pointer
/// and `mov` element into stack)
///
/// # Fields
///
/// `register`: This is the register that is being pushed onto the stack. The type
/// of the register is generic (`T`), so it can be defined to fit the particular
/// architecture or simulation you're working with.
///
/// # Example
///
/// The `PushOperation` can be used to represent assembly `push` instructions.
/// For instance, in x86 assembly, the instruction `push eax` would push the value
/// of the `eax` register onto the stack. This can be modeled using `PushOperation`
/// as follows:
///
/// ```
/// use reloaded_hooks_portable::api::jit::push_operation::PushOperation;
/// let push_eax = PushOperation { register: "eax" };
/// ```
///
/// Similarly, in architectures without an explicit `push` instruction, this operation
/// can be modeled as a subtraction (`sub`) operation on the stack pointer (to allocate
/// space on the stack), followed by a `mov` operation to move the register value into
/// the newly allocated space. For example, in ARM assembly, `push r1` could be
/// represented as:
///
/// ```
/// use reloaded_hooks_portable::api::jit::stack_alloc_operation::StackAllocOperation;
/// use reloaded_hooks_portable::api::jit::mov_operation::MovOperation;
///
/// let sub_sp = StackAllocOperation { operand: 4 }; // assuming 4 bytes per register
/// let mov_into_stack = MovOperation { source: "r1", target: "[sp]" };
/// ```
///
/// In these cases, the `PushOperation` can be used as an abstraction of these two
/// operations, simplifying the representation of the operation and the underlying
/// computation or analysis you're performing.
///
/// # Note
///
/// The `PushOperation` only represents the operation itself; it does not perform
/// the operation or modify any actual register or memory values. To simulate the
/// effect of the operation, you would need to implement additional logic or use
/// a computing architecture simulation framework.
///
/// In the real world, you should use enums instead of strings for source and target 😉,
/// the code above shows strings for clarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, new)]
pub struct PushOperation<T: Copy> {
    /// The register to push onto the stack.
    pub register: T,
}
