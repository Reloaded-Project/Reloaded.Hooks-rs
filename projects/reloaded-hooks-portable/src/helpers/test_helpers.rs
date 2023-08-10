use crate::api::{
    function_attribute::{FunctionAttribute, StackCleanup, StackParameterOrder},
    function_info::{FunctionInfo, ParameterType},
    traits::register_info::RegisterInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MockRegister {
    R1,
    R2,
    R3,
    R4,
    F1,
    F2,
    F3,
    F4,
    SP,
}

impl RegisterInfo for MockRegister {
    fn size_in_bytes(&self) -> usize {
        match self {
            MockRegister::R1 => 4,
            MockRegister::R2 => 4,
            MockRegister::R3 => 4,
            MockRegister::R4 => 4,
            MockRegister::F1 => 4,
            MockRegister::F2 => 4,
            MockRegister::F3 => 4,
            MockRegister::F4 => 4,
            MockRegister::SP => 4,
        }
    }

    fn is_stack_pointer(&self) -> bool {
        self == &MockRegister::SP
    }

    fn register_type(&self) -> usize {
        match self {
            MockRegister::R1 => 0,
            MockRegister::R2 => 0,
            MockRegister::R3 => 0,
            MockRegister::R4 => 0,
            MockRegister::SP => 0,

            MockRegister::F1 => 1,
            MockRegister::F2 => 1,
            MockRegister::F3 => 1,
            MockRegister::F4 => 1,
        }
    }
}

pub struct MockFunctionAttribute {
    pub int_params: Vec<MockRegister>,
    pub float_params: Vec<MockRegister>,
    pub return_reg: MockRegister,
    pub reserved_stack: u32,
    pub callee_saved: Vec<MockRegister>,
    pub always_saved: Vec<MockRegister>,
    pub stack_cleanup: StackCleanup,
    pub stack_param_order: StackParameterOrder,
}

impl Default for MockFunctionAttribute {
    fn default() -> Self {
        MockFunctionAttribute {
            int_params: vec![],
            float_params: vec![],
            return_reg: MockRegister::R1,
            reserved_stack: 0,
            callee_saved: vec![],
            always_saved: vec![],
            stack_cleanup: StackCleanup::Caller,
            stack_param_order: StackParameterOrder::RightToLeft,
        }
    }
}

impl FunctionAttribute<MockRegister> for MockFunctionAttribute {
    fn register_int_parameters(&self) -> &[MockRegister] {
        &self.int_params
    }

    fn register_float_parameters(&self) -> &[MockRegister] {
        &self.float_params
    }

    fn return_register(&self) -> MockRegister {
        MockRegister::R1
    }

    fn reserved_stack_space(&self) -> u32 {
        0
    }

    fn callee_saved_registers(&self) -> &[MockRegister] {
        &[]
    }

    fn always_saved_registers(&self) -> &[MockRegister] {
        &[]
    }

    fn stack_cleanup_behaviour(&self) -> StackCleanup {
        StackCleanup::Caller
    }

    fn stack_parameter_order(&self) -> StackParameterOrder {
        StackParameterOrder::RightToLeft
    }
}

pub struct MockFunction {
    pub parameters: Vec<ParameterType>,
}

impl FunctionInfo for MockFunction {
    fn parameters(&self) -> &[ParameterType] {
        &self.parameters
    }
}
