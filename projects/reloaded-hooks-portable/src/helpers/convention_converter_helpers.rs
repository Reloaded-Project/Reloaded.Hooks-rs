use crate::api::{function_attribute::FunctionAttribute, function_info::FunctionInfo};

/// This function returns true if the function can reuse stack parameters
/// during conversion between two individual calling conventions.
///
/// # Parameters
/// - `source`: The calling convention of the source function. (The function called by the wrapper)
/// - `target`: The calling convention of the target function. (The function returned)
/// - `info`: The information about the function being wrapped.
///
/// # Returns
///
/// True if both calling convention produce the same sequence of stack spilled parameters,
/// else false.
pub fn can_reuse_stack_parameters<
    TRegister,
    T: FunctionAttribute<TRegister>,
    TInfo: FunctionInfo,
>(
    source_convention: &T,
    target_convention: &T,
    info: &TInfo,
) -> bool {
    let stack_params_source = info.get_stack_parameters(source_convention);
    let stack_params_target = info.get_stack_parameters(target_convention);
    stack_params_source == stack_params_target
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{api::function_info::ParameterType, helpers::test_helpers::*};

    #[test]
    fn can_reuse_stack_parameters_same_spilled() {
        let mock_func_info = MockFunction {
            parameters: vec![ParameterType::i32, ParameterType::i64, ParameterType::f32],
        };

        let mock_attr1 = MockFunctionAttribute {
            int_params: vec![MockRegister::R1],
            float_params: vec![MockRegister::F1],
            ..Default::default()
        };

        let mock_attr2 = MockFunctionAttribute {
            int_params: vec![MockRegister::R1],
            float_params: vec![MockRegister::F1],
            ..Default::default()
        };

        assert!(can_reuse_stack_parameters(
            &mock_attr1,
            &mock_attr2,
            &mock_func_info,
        ));
    }

    #[test]
    fn test_can_reuse_stack_parameters_different_spilled() {
        let mock_func_info = MockFunction {
            parameters: vec![ParameterType::i32, ParameterType::i64, ParameterType::f32],
        };

        let mock_attr1 = MockFunctionAttribute {
            int_params: vec![MockRegister::R1],
            float_params: vec![MockRegister::F1],
            ..Default::default()
        };

        let mock_attr2 = MockFunctionAttribute {
            int_params: vec![MockRegister::R1, MockRegister::R2],
            float_params: vec![MockRegister::F1],
            ..Default::default()
        };

        assert!(!can_reuse_stack_parameters(
            &mock_attr1,
            &mock_attr2,
            &mock_func_info
        ));
    }

    #[test]
    fn test_can_reuse_stack_parameters_no_spilled() {
        let mock_func_info = MockFunction {
            parameters: vec![ParameterType::i32],
        };

        let mock_attr1 = MockFunctionAttribute {
            int_params: vec![MockRegister::R1],
            float_params: vec![],
            ..Default::default()
        };

        let mock_attr2 = MockFunctionAttribute {
            int_params: vec![MockRegister::R1],
            float_params: vec![],
            ..Default::default()
        };

        assert!(can_reuse_stack_parameters(
            &mock_attr1,
            &mock_attr2,
            &mock_func_info
        ));
    }
}
