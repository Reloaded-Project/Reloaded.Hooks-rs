extern crate alloc;

use core::mem::size_of;

use super::function_attribute::FunctionAttribute;
use alloc::vec::Vec;

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

    /// Returns the parameters that would be put to the stack if this
    /// function were to be used with the specified calling convention.
    fn get_stack_parameters<TRegister, T: FunctionAttribute<TRegister>>(
        &self,
        convention: &T,
    ) -> Vec<ParameterType> {
        let parameters = self.parameters();
        let mut result = Vec::<ParameterType>::with_capacity(self.parameters().len());
        let mut num_int_params = convention.register_int_parameters().len() as i32;
        let mut num_float_params = convention.register_float_parameters().len() as i32;
        let mut num_vector_params = convention.register_vector_parameters().len() as i32;

        for &parameter in parameters {
            if parameter.is_float() {
                // Check if we still have any float registers available, if not, push to stack
                if num_float_params > 0 {
                    num_float_params -= 1;
                } else {
                    result.push(parameter);
                }
            } else if parameter.is_vector() {
                // Check if we still have any vector registers available, if not, push to stack
                if num_vector_params > 0 {
                    num_vector_params -= 1;
                } else {
                    result.push(parameter);
                }
            } else {
                // Check if we still have any int registers available, if not, push to stack
                if num_int_params > 0 {
                    num_int_params -= 1;
                } else {
                    result.push(parameter);
                }
            }
        }

        result
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

    /// Represents a 16-bit floating-point number that sits in a specialized register.
    /// Note: Use only on architectures that have explicit matrix/vector registers like MIPS or
    /// possibly RISC-V in the future.
    v16,

    /// Represents a 32-bit floating-point number that sits in a specialized register.
    /// Note: Use only on architectures that have explicit matrix/vector registers like MIPS or
    /// possibly RISC-V in the future.
    v32,

    /// Represents a 64-bit floating-point number that sits in a specialized register.
    /// Note: Use only on architectures that have explicit matrix/vector registers like MIPS or
    /// possibly RISC-V in the future.
    v64,

    /// Represents a 128-bit floating-point number that sits in a specialized register.
    /// Note: Use only on architectures that have explicit matrix/vector registers like MIPS or
    /// possibly RISC-V in the future.
    v128,

    /// Represents a 256-bit floating-point number that sits in a specialized register.
    /// Note: Use only on architectures that have explicit matrix/vector registers like MIPS or
    /// possibly RISC-V in the future.
    v256,

    /// Represents a 512-bit floating-point number that sits in a specialized register.
    /// Note: Use only on architectures that have explicit matrix/vector registers like MIPS or
    /// possibly RISC-V in the future.
    v512,
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
        !self.is_float() && !self.is_vector()
    }

    /// Determines if the parameter is a vector type.
    pub fn is_vector(&self) -> bool {
        matches!(
            *self,
            ParameterType::v16
                | ParameterType::v32
                | ParameterType::v64
                | ParameterType::v128
                | ParameterType::v256
                | ParameterType::v512
        )
    }

    /// Returns the size in bytes of the parameter type.
    pub fn size_in_bytes(&self) -> usize {
        match *self {
            ParameterType::nint => size_of::<isize>(),
            ParameterType::i8 | ParameterType::v16 | ParameterType::f16 => 1,
            ParameterType::i16 | ParameterType::v32 | ParameterType::f32 => 2,
            ParameterType::i32 | ParameterType::v64 | ParameterType::f64 => 4,
            ParameterType::i64 | ParameterType::v128 | ParameterType::f128 => 8,
            ParameterType::i128 | ParameterType::v256 | ParameterType::f256 => 16,
            ParameterType::v512 | ParameterType::f512 => 64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::test_helpers::*;

    fn create_mock_function(params: Vec<ParameterType>) -> MockFunction {
        MockFunction { parameters: params }
    }

    fn create_mock_attribute(
        int_params: Vec<MockRegister>,
        float_params: Vec<MockRegister>,
    ) -> MockFunctionAttribute {
        MockFunctionAttribute {
            int_params,
            float_params,
            ..Default::default()
        }
    }

    fn create_mock_attribute_with_vec(
        int_params: Vec<MockRegister>,
        vector_params: Vec<MockRegister>,
    ) -> MockFunctionAttribute {
        MockFunctionAttribute {
            int_params,
            vector_params,
            ..Default::default()
        }
    }

    fn get_spilled_parameters(
        function: &MockFunction,
        attribute: &MockFunctionAttribute,
    ) -> Vec<ParameterType> {
        function.get_stack_parameters(attribute).to_vec()
    }

    #[test]
    fn get_stack_parameters_with_float_spilled() {
        let function = create_mock_function(vec![
            ParameterType::i32,
            ParameterType::f32,
            ParameterType::f64,
            ParameterType::i64,
        ]);
        let attribute = create_mock_attribute(
            vec![MockRegister::R1, MockRegister::R2],
            vec![MockRegister::F1],
        );

        assert_eq!(
            get_spilled_parameters(&function, &attribute),
            vec![ParameterType::f64]
        );
    }

    #[test]
    fn get_stack_parameters_with_vector_spilled() {
        let function = create_mock_function(vec![
            ParameterType::i32,
            ParameterType::v32,
            ParameterType::v64,
            ParameterType::i64,
        ]);
        let attribute = create_mock_attribute_with_vec(
            vec![MockRegister::R1, MockRegister::R2],
            vec![MockRegister::V1],
        );

        assert_eq!(
            get_spilled_parameters(&function, &attribute),
            vec![ParameterType::v64]
        );
    }

    #[test]
    fn get_stack_parameters_with_no_spilled() {
        let function = create_mock_function(vec![
            ParameterType::i32,
            ParameterType::f32,
            ParameterType::f64,
            ParameterType::i64,
        ]);
        let attribute = create_mock_attribute(
            vec![MockRegister::R1, MockRegister::R2],
            vec![MockRegister::F1, MockRegister::F2],
        );

        assert_eq!(get_spilled_parameters(&function, &attribute), vec![]);
    }

    #[test]
    fn get_stack_parameters_int_spilled() {
        let function = create_mock_function(vec![
            ParameterType::i32,
            ParameterType::i32,
            ParameterType::i32,
            ParameterType::i32,
            ParameterType::i32,
        ]);
        let attribute = create_mock_attribute(
            vec![
                MockRegister::R1,
                MockRegister::R2,
                MockRegister::R3,
                MockRegister::R4,
            ],
            vec![],
        );

        assert_eq!(
            get_spilled_parameters(&function, &attribute),
            vec![ParameterType::i32]
        );
    }

    #[test]
    fn get_stack_parameters_float_spilled() {
        let function = create_mock_function(vec![
            ParameterType::f32,
            ParameterType::f32,
            ParameterType::f32,
            ParameterType::f32,
            ParameterType::f32,
        ]);
        let attribute = create_mock_attribute(
            vec![],
            vec![
                MockRegister::F1,
                MockRegister::F2,
                MockRegister::F3,
                MockRegister::F4,
            ],
        );

        assert_eq!(
            get_spilled_parameters(&function, &attribute),
            vec![ParameterType::f32]
        );
    }
}
