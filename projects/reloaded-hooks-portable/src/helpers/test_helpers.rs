use crate::api::{
    function_attribute::{FunctionAttribute, StackCleanup, StackParameterOrder},
    function_info::{FunctionInfo, ParameterType},
    traits::register_info::RegisterInfo,
};
use lazy_static;

#[allow(dead_code)]
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
    V1,
    V2,
    V3,
    V4,
    SP,

    // This is used when we test with pretend-architectures that don't use stack for return address.
    LR,
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
            MockRegister::V1 => 4,
            MockRegister::V2 => 4,
            MockRegister::V3 => 4,
            MockRegister::V4 => 4,
            MockRegister::SP => 4,
            MockRegister::LR => 4,
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

            MockRegister::LR => 2,

            MockRegister::V1 => 3,
            MockRegister::V2 => 3,
            MockRegister::V3 => 3,
            MockRegister::V4 => 3,
        }
    }
}

pub struct MockFunctionAttribute {
    pub int_params: Vec<MockRegister>,
    pub float_params: Vec<MockRegister>,
    pub vector_params: Vec<MockRegister>,
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
            vector_params: vec![],
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

    fn register_vector_parameters(&self) -> &[MockRegister] {
        &self.vector_params
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

    fn required_stack_alignment(&self) -> usize {
        0
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

// Testing Calling Conventions
lazy_static::lazy_static! {

    /// A calling convention that is similar to x86 'cdecl', but for our pretend architecture.
    pub static ref CDECL_LIKE_FUNCTION_ATTRIBUTE: MockFunctionAttribute = MockFunctionAttribute {
        int_params: vec![],
        float_params: vec![],
        vector_params: vec![],
        return_reg: MockRegister::R1,
        reserved_stack: 0,
        callee_saved: vec![MockRegister::R3, MockRegister::R4],
        always_saved: vec![],
        stack_cleanup: StackCleanup::Caller,
        stack_param_order: StackParameterOrder::RightToLeft,
    };

    /// A calling convention that is similar to x86 'stdcall', but for our pretend architecture.
    pub static ref STDCALL_LIKE_FUNCTION_ATTRIBUTE: MockFunctionAttribute = MockFunctionAttribute {
        int_params: vec![],
        float_params: vec![],
        vector_params: vec![],
        return_reg: MockRegister::R1,
        reserved_stack: 0,
        callee_saved: vec![MockRegister::R3, MockRegister::R4],
        always_saved: vec![],
        stack_cleanup: StackCleanup::Callee,  // callee cleanup
        stack_param_order: StackParameterOrder::RightToLeft,
    };

    /// A calling convention that is similar to x86 Microsoft 'thiscall', but for our pretend architecture.
    pub static ref THISCALL_LIKE_FUNCTION_ATTRIBUTE: MockFunctionAttribute = MockFunctionAttribute {
        int_params: vec![ MockRegister::R1 ], // R1 is 'this' pointer
        float_params: vec![],
        vector_params: vec![],
        return_reg: MockRegister::R1,
        reserved_stack: 0,
        callee_saved: vec![MockRegister::R3, MockRegister::R4],
        always_saved: vec![],
        stack_cleanup: StackCleanup::Callee,  // callee cleanup
        stack_param_order: StackParameterOrder::RightToLeft,
    };
}
