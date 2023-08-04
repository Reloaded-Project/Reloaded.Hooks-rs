/// Represents a pop operation which retrieves a value from the top of the stack,
/// stores it in a register and decrements the current stack pointer.
///
/// This can be used to model assembly `pop` instructions.
/// (Or in cases of architectures without an explicit `pop` instruction, `mov` element from stack
/// and `add` to stack pointer)
///
/// # Fields
///
/// `register`: This is the register that the value is being popped into from the stack. The type
/// of the register is generic (`T`), so it can be defined to fit the particular
/// architecture or simulation you're working with.
///
/// # Example
///
/// The `PopOperation` can be used to represent assembly `pop` instructions.
/// For instance, in x86 assembly, the instruction `pop eax` would retrieve the value
/// from the top of the stack and store it into the `eax` register. This can be modeled using `PopOperation`
/// as follows:
///
/// ```
/// use reloaded_hooks_portable::structs::pop_operation::PopOperation;
/// let pop_eax = PopOperation { register: "eax" };
/// ```
///
/// Similarly, in architectures without an explicit `pop` instruction, this operation
/// can be modeled as a `mov` operation to move the value from the stack into the register,
/// followed by an addition (`add`) operation on the stack pointer (to deallocate
/// space on the stack). For example, in ARM assembly, `pop r1` could be
/// represented as:
///
/// ```
/// use reloaded_hooks_portable::structs::mov_operation::MovOperation;
/// use reloaded_hooks_portable::structs::sub_operation::SubOperation;
///
/// let mov_from_stack = MovOperation { source: "[sp]", target: "r1" };
/// let add_sp = SubOperation { register: "sp", operand: -4 }; // assuming 4 bytes per register
/// ```
///
/// In these cases, the `PopOperation` can be used as an abstraction of these two
/// operations, simplifying the representation of the operation and the underlying
/// computation or analysis you're performing.
///
/// # Note
///
/// The `PopOperation` only represents the operation itself; it does not perform
/// the operation or modify any actual register or memory values. To simulate the
/// effect of the operation, you would need to implement additional logic or use
/// a computing architecture simulation framework.
///
/// In the real world, you should use enums instead of strings for source and target 😉,
/// the code above shows strings for clarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubOperation<T> {
    /// The register from which the value will be subtracted.
    pub register: T,
    /// The value to subtract from the register.
    pub operand: i32,
}
