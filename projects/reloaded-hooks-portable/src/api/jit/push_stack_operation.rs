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
    /// Creates a new `PushStackOperation` with the given offset and item size.
    ///
    /// # Examples
    ///
    /// ```
    /// use reloaded_hooks_portable::api::jit::push_stack_operation::PushStackOperation;
    ///
    /// let push_op = PushStackOperation::<i32>::with_offset_and_size(0, 4);
    /// assert_eq!(push_op.offset, 0);
    /// assert_eq!(push_op.item_size, 4);
    /// assert_eq!(push_op.scratch_1, None);
    /// assert_eq!(push_op.scratch_2, None);
    /// ```
    pub fn with_offset_and_size(offset: i32, item_size: u32) -> Self {
        Self {
            offset,
            item_size,
            scratch_1: None,
            scratch_2: None,
        }
    }

    /// Returns the number of scratch registers used in the push operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use reloaded_hooks_portable::api::jit::push_stack_operation::PushStackOperation;
    ///
    /// let push_op = PushStackOperation::<i32>::with_offset_and_size(0, 4);
    /// assert_eq!(push_op.num_scratch_registers(), 0);
    ///
    /// let push_op = PushStackOperation::<i32>::with_scratch_registers(0, 4, &[1]);
    /// assert_eq!(push_op.num_scratch_registers(), 1);
    ///
    /// let push_op = PushStackOperation::<i32>::with_scratch_registers(0, 4, &[1, 2]);
    /// assert_eq!(push_op.num_scratch_registers(), 2);
    /// ```
    pub fn num_scratch_registers(&self) -> usize {
        let mut count = 0;
        if self.scratch_1.is_some() {
            count += 1;
        }
        if self.scratch_2.is_some() {
            count += 1;
        }
        count
    }

    /// Returns true if this operation has the provided offset and size.
    ///
    /// # Parameters
    /// - `offset`: The offset to check against.
    /// - `item_size`: The item size to check against.
    ///
    /// # Returns
    /// Returns `true` if the `offset` and `item_size` of the operation match the provided values,
    /// otherwise returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use reloaded_hooks_portable::api::jit::push_stack_operation::PushStackOperation;
    ///
    /// let push_op = PushStackOperation::<i32>::with_offset_and_size(0, 4);
    /// assert!(push_op.has_offset_and_size(0, 4));  // This will return true
    /// assert!(!push_op.has_offset_and_size(1, 4));  // This will return false
    /// assert!(!push_op.has_offset_and_size(0, 2));  // This will return false
    /// ```
    pub fn has_offset_and_size(&self, offset: i32, item_size: u32) -> bool {
        self.offset == offset && self.item_size == item_size
    }
}

impl<T: Copy> PushStackOperation<T> {
    /// Creates a new `PushStackOperation` with scratch registers.
    ///
    /// # Examples
    ///
    /// ```
    /// use reloaded_hooks_portable::api::jit::push_stack_operation::PushStackOperation;
    ///
    /// let push_op = PushStackOperation::<i32>::with_scratch_registers(0, 4, &[1, 2]);
    /// assert_eq!(push_op.offset, 0);
    /// assert_eq!(push_op.item_size, 4);
    /// assert_eq!(push_op.scratch_1, Some(1));
    /// assert_eq!(push_op.scratch_2, Some(2));
    /// ```
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
