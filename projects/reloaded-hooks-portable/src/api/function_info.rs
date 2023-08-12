extern crate alloc;

/// This trait defines the information about the function for which a wrapper is being generated.
///
/// # Generic Parameters
/// - `TRegister`: The type of register used by the target architecture. (Enum)
pub trait FunctionInfo {
    /// Types of parameters in left-right order.
    fn parameters(&self) -> &[ParameterType];

    /// Returns the number of integer parameters in the function.
    fn num_integer_parameters(&self) -> u32 {
        self.parameters()
            .iter()
            .filter(|&param| {
                matches!(
                    param,
                    ParameterType::nint
                        | ParameterType::i8
                        | ParameterType::i16
                        | ParameterType::i32
                        | ParameterType::i64
                        | ParameterType::i128
                )
            })
            .count() as u32
    }

    /// Returns the number of float parameters in the function.
    fn num_float_parameters(&self) -> u32 {
        self.parameters()
            .iter()
            .filter(|&param| {
                matches!(
                    param,
                    ParameterType::f16
                        | ParameterType::f32
                        | ParameterType::f64
                        | ParameterType::f128
                        | ParameterType::f256
                        | ParameterType::f512
                )
            })
            .count() as u32
    }
}

/// Defines the kind of parameter used in the function.
///
/// # Usage Guidance
///
/// For pointers and memory addresses use `nint`.
///
/// If you are unsure about any variable, use the closest approximation you believe your type
/// falls under. For example, if you have a type called AtomicI32, and all it
/// contains inside is a 32-bit integer, you should use `i32` as the parameter kind.
///  
/// # Remarks
///
/// This enumerable is defined with future extensibility in mind; especially
/// supporting newer architectures like RISC-V, Intel extensions and ARM64.
/// Today, all of these entries are most likely internally categorised as 'float'
/// or 'integer', by the library.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
    /// Represents a native sized integer, use this for pointers.
    nint,

    /// Represents an 8-bit signed integer.
    i8,

    /// Represents a 16-bit signed integer.
    i16,

    /// Represents a 32-bit signed integer.
    i32,

    /// Represents a 64-bit signed integer.
    i64,

    /// Represents a 128-bit signed integer.
    i128,

    /// Represents a 16-bit floating-point number.
    f16,

    /// Represents a 32-bit floating-point number.
    f32,

    /// Represents a 64-bit floating-point number.
    f64,

    /// Represents a 128-bit floating-point number (often referred to as a "quad precision" float).
    f128,

    /// Represents a 256-bit floating-point number. Used for certain vectorized operations in some architectures.
    f256,

    /// Represents a 512-bit floating-point number. Commonly associated with AVX-512 on Intel architectures.
    f512,
}

/// Extension methods for ParameterType enum.
impl ParameterType {
    /// Determines if the parameter is a floating-point type.
    pub fn is_float(&self) -> bool {
        matches!(
            *self,
            ParameterType::f16
                | ParameterType::f32
                | ParameterType::f64
                | ParameterType::f128
                | ParameterType::f256
                | ParameterType::f512
        )
    }

    /// Determines if the parameter is an integer type.
    pub fn is_integer(&self) -> bool {
        !self.is_float()
    }
}
