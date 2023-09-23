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
/// let push_offset_from_esp = PushStackOperation::<i32>::with_offset_and_size(8, 4);
/// ```
///
/// # Remarks
///
/// Number of copied bytes should be a multiple of native register size.
/// Other values may not be supported depending on implementation.
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, new)]
pub struct PushStackOperation<T> {
    /// The offset from the current stack pointer in the direction opposite to the stack's growth.
    pub offset: i32,

    /// Size of the item to re-push to stack.
    pub item_size: u32,

    /// Scratch register to use for the push operation. (Needed for some architectures)
    pub scratch_1: Option<T>,

    /// Scratch register to use for the push operation. (Needed for some architectures)
    pub scratch_2: Option<T>,
}

impl<T> Default for PushStackOperation<T> {
    fn default() -> Self {
        Self {
            offset: Default::default(),
            item_size: Default::default(),
            scratch_1: Default::default(),
            scratch_2: Default::default(),
        }
    }
}

impl<T> PushStackOperation<T> {
    pub fn with_offset_and_size(offset: i32, item_size: u32) -> Self {
        Self {
            offset,
            item_size,
            scratch_1: None,
            scratch_2: None,
        }
    }
}

impl<T: Copy> PushStackOperation<T> {
    pub fn with_scratch_registers(offset: i32, item_size: u32, registers: &[T]) -> Self {
        let mut me = Self::with_offset_and_size(offset, item_size);
        if registers.len() > 1 {
            me.scratch_1 = Some(registers[0]);
            me.scratch_2 = Some(registers[1]);
        } else if !registers.is_empty() {
            me.scratch_1 = Some(registers[0]);
        }

        me
    }
}
