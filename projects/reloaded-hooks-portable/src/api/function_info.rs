extern crate alloc;

use super::{calling_convention_info::CallingConventionInfo, traits::register_info::RegisterInfo};
use alloc::vec::Vec;
use core::mem::size_of;
use derive_new::new;

/// This trait defines the information about the function for which a wrapper is being generated.
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

    /// Returns the parameters that would be put to the stack and heap if the
    /// function were to be used with the specified calling convention.
    ///
    /// # Parameters
    /// - `convention`: The calling convention to use.
    /// - `stack_params`: A mutable slice of parameters that will be put on the stack.
    ///                   This slice must be at least [`FunctionInfo::parameters()`].len() in length.
    ///
    /// - `reg_params`: A mutable slice of parameters that will be put in registers.
    ///                 This slice must be at least [`FunctionInfo::parameters()`].len() in length.
    ///
    /// # Returns
    ///
    /// Tuple of (stack parameters, register parameters)
    /// These are the original passed in slices, sliced to contain just the filled in elements.
    fn get_parameters_as_slice<
        'a,
        TRegister: Clone + Copy + RegisterInfo + PartialEq + 'static,
        T: CallingConventionInfo<TRegister>,
    >(
        &self,
        convention: &T,
        stack_params: &'a mut [ParameterType], // Mutable slice for stack parameters
        reg_params: &'a mut [(ParameterType, TRegister)], // Mutable slice for register parameters
    ) -> (
        &'a mut [ParameterType],
        &'a mut [(ParameterType, TRegister)],
    ) {
        let parameters = self.parameters();

        let mut stack_idx = 0;
        let mut reg_idx = 0;

        let mut int_registers = convention.register_int_parameters().iter();
        let mut float_registers = convention.register_float_parameters().iter();
        let mut vector_registers = convention.register_vector_parameters().iter();

        for &parameter in parameters {
            if parameter.is_float() {
                if let Some(reg) = float_registers.next() {
                    reg_params[reg_idx] = (parameter, *reg);
                    reg_idx += 1;
                } else {
                    stack_params[stack_idx] = parameter;
                    stack_idx += 1;
                }
            } else if parameter.is_vector() {
                if let Some(reg) = vector_registers.next() {
                    reg_params[reg_idx] = (parameter, *reg);
                    reg_idx += 1;
                } else {
                    stack_params[stack_idx] = parameter;
                    stack_idx += 1;
                }
            } else if let Some(reg) = int_registers.next() {
                reg_params[reg_idx] = (parameter, *reg);
                reg_idx += 1;
            } else {
                stack_params[stack_idx] = parameter;
                stack_idx += 1;
            }
        }

        // Return slices that match the populated portions
        (&mut stack_params[0..stack_idx], &mut reg_params[0..reg_idx])
    }

    /// Returns the parameters that would be put to the stack and heap if the
    /// function were to be used with the specified calling convention.
    ///
    /// # Parameters
    /// - `convention`: The calling convention to use.
    /// - `stack_params`: A mutable slice of parameters that will be put on the stack.
    ///                   This slice must be at least `self.parameters.len()` in length.
    ///
    /// - `reg_params`: A mutable slice of parameters that will be put in registers.
    ///                 This slice must be at least `self.parameters.len()` in length.
    ///
    /// # Returns
    ///
    /// Tuple of (stack parameters, register parameters) as vectors
    /// These are the original passed in slices, sliced to contain just the filled in elements.
    fn get_parameters_as_vec<
        TRegister: Clone + Copy + RegisterInfo + PartialEq + 'static,
        T: CallingConventionInfo<TRegister>,
    >(
        &self,
        convention: &T,
    ) -> (Vec<ParameterType>, Vec<(ParameterType, TRegister)>) {
        let parameters = self.parameters();

        let mut stack_params = Vec::with_capacity(parameters.len());
        let mut reg_params = Vec::with_capacity(parameters.len());
        unsafe {
            stack_params.set_len(stack_params.capacity());
            reg_params.set_len(reg_params.capacity());
        }

        let (filled_stack, filled_reg) =
            self.get_parameters_as_slice(convention, &mut stack_params, &mut reg_params);

        let filled_stack_len = filled_stack.len();
        let filled_reg_len = filled_reg.len();
        stack_params.truncate(filled_stack_len);
        reg_params.truncate(filled_reg_len);

        (stack_params, reg_params)
    }
}

/// Basic reference implementation of [`FunctionInfo`]
#[derive(new)]
pub struct BasicFunctionInfo<'a> {
    params: &'a [ParameterType],
}

impl<'a> FunctionInfo for BasicFunctionInfo<'a> {
    fn parameters(&self) -> &[ParameterType] {
        self.params
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
        function.get_parameters_as_vec(attribute).0
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
