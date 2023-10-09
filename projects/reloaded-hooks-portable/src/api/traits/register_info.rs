/// Implemented by register types which need to express their size in bytes to the world.
pub trait RegisterInfo {
    /// Returns the size of the register in bytes.
    fn size_in_bytes(&self) -> usize;

    /// True if the register is a stack pointer.
    fn is_stack_pointer(&self) -> bool;

    /// Extends the register to the maximum size of this specific register available on this system.
    ///
    /// The `extend` method is responsible for converting a register representation to its largest
    /// form available. For instance, in a 64-bit x86 architecture, calling `extend` on an object
    /// representing `EAX` (a 32-bit register) would return an object representing `RAX`
    /// (a 64-bit register).
    ///
    /// # Examples
    ///
    /// (Note: Not rustdoc typed because no arch specific stuff in this package)
    ///
    /// ```ignore
    /// let eax = Register::EAX;
    /// let rax = eax.extend(); // now we have "rax"
    /// ```
    ///
    /// On a system without EAX.
    ///
    /// # Remarks
    ///
    /// This method is particularly useful in scenarios where operations need to be performed
    /// with registers in their extended forms to leverage the full data capacity. The actual
    /// implementation may vary depending on the architecture and register types, ensuring the
    /// correct handling of different register varieties and their extensions.
    fn extend(&self) -> Self;

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
    fn register_type(&self) -> KnownRegisterType;

    /// Retrieves all of the available values for the current register type.
    ///
    /// # Returns
    ///
    /// All possible values for the type implementing RegisterInfo.
    /// ```
    fn all_registers() -> &'static [Self]
    where
        Self: Sized;

    /// Finds a register with the same type as the given register.
    ///
    /// # Arguments
    ///
    /// * `available_registers` - The slice of available registers to search through.
    ///
    /// # Returns
    ///
    /// Returns the first register with the same type as the given register, or `None` if no match is found.
    /// ```
    fn find_register_with_same_type<TRegister: Copy + RegisterInfo>(
        &self,
        available_registers: &[TRegister],
    ) -> Option<TRegister> {
        let expected_type = self.register_type();
        for register in available_registers {
            if register.register_type() == expected_type {
                return Some(*register);
            }
        }

        None
    }
}

/// Finds a register with a given category.
///
/// # Arguments
///
/// * `category` - The category of register to search for.
/// * `available_registers` - The slice of available registers to search through.
///
/// # Returns
///
/// Returns the first register with the same type as the given register, or `None` if no match is found.
/// ```
pub(crate) fn find_register_with_category<TRegister: Copy + RegisterInfo>(
    category: RegisterCategory,
    available_registers: &[TRegister],
) -> Option<TRegister> {
    for register in available_registers {
        if register.register_type().category() == category {
            return Some(*register);
        }
    }

    None
}

/// Enum representing different known register types.
///
/// This enum is used to differentiate between different types of registers
/// available in a computer architecture. Different register types are used
/// for different kinds of operations and data.
///
/// # Remarks
///
/// The values are assigned such that you can figure out the category of the register using bit
/// flag comparisons.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KnownRegisterType {
    /// An unknown register type.
    Unknown = 0,

    /// A 128-bit general purpose register.
    ///
    /// General purpose registers are used to store temporary data during
    /// the execution of a program and are used for various integer operations.
    GeneralPurpose128 = 0b1000,

    /// A 64-bit general purpose register.
    ///
    /// Suitable for storing addresses and integers on 64-bit architectures.
    GeneralPurpose64,

    /// A 32-bit general purpose register.
    ///
    /// Commonly used in 32-bit and 64-bit architectures for integer operations.
    GeneralPurpose32,

    /// A 16-bit general purpose register.
    ///
    /// Used for smaller integer values, particularly in older 16-bit architectures.
    GeneralPurpose16,

    /// An 8-bit general purpose register.
    ///
    /// Used for byte-sized integer values.
    GeneralPurpose8,

    /// A floating-point register.
    ///
    /// Used to store floating-point numbers and perform floating-point arithmetic.
    /// For example, x87 registers in x86 architecture.
    FloatingPoint = 0b10000,

    /// A 512-bit vector register.
    ///
    /// Suitable for SIMD (Single Instruction, Multiple Data) operations,
    /// such as those available with AVX512 instructions in x86_64 architecture.
    Vector512 = 0b100000,

    /// A 256-bit vector register.
    ///
    /// Used for SIMD operations, commonly available in modern CPU architectures.
    Vector256,

    /// A 128-bit vector register.
    ///
    /// Suitable for SIMD operations and commonly used for multimedia instructions.
    Vector128,

    /// A 64-bit vector register.
    ///
    /// Used for smaller SIMD operations.
    Vector64,

    /// A 32-bit vector register.
    ///
    /// Suitable for SIMD operations on small data types like bytes and short integers.
    Vector32,
}

impl KnownRegisterType {
    /// Determines the high-level category of the register type.
    ///
    /// # Returns
    ///
    /// * `GeneralPurpose` - if the register is a general-purpose register.
    /// * `FloatingPoint` - if the register is a floating-point register.
    /// * `Vector` - if the register is a vector register.
    pub fn category(&self) -> RegisterCategory {
        let value = *self as u32;
        if value & 0b1000 != 0 {
            RegisterCategory::GeneralPurpose
        } else if value & 0b10000 != 0 {
            RegisterCategory::FloatingPoint
        } else if value & 0b100000 != 0 {
            RegisterCategory::Vector
        } else {
            RegisterCategory::Unknown
        }
    }
}

/// Enum representing the high-level category of a register.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RegisterCategory {
    Unknown,
    GeneralPurpose,
    FloatingPoint,
    Vector,
}
