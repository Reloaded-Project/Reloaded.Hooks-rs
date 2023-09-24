extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use crate::api::{
    calling_convention_info::*,
    function_info::{FunctionInfo, ParameterType},
    traits::register_info::{KnownRegisterType, KnownRegisterType::*, RegisterInfo},
};
use lazy_static;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MockRegister {
    #[default]
    R0,
    R1,
    R2,
    R3,
    R4,
    F0,
    F1,
    F2,
    F3,
    F4,
    V0,
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
            MockRegister::R0 => 4,
            MockRegister::R1 => 4,
            MockRegister::R2 => 4,
            MockRegister::R3 => 4,
            MockRegister::R4 => 4,
            MockRegister::F0 => 4,
            MockRegister::F1 => 4,
            MockRegister::F2 => 4,
            MockRegister::F3 => 4,
            MockRegister::F4 => 4,
            MockRegister::V0 => 4,
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

    fn register_type(&self) -> KnownRegisterType {
        match self {
            MockRegister::R0 => GeneralPurpose64,
            MockRegister::R1 => GeneralPurpose64,
            MockRegister::R2 => GeneralPurpose64,
            MockRegister::R3 => GeneralPurpose64,
            MockRegister::R4 => GeneralPurpose64,
            MockRegister::SP => GeneralPurpose64,
            MockRegister::LR => GeneralPurpose64,

            MockRegister::F0 => FloatingPoint,
            MockRegister::F1 => FloatingPoint,
            MockRegister::F2 => FloatingPoint,
            MockRegister::F3 => FloatingPoint,
            MockRegister::F4 => FloatingPoint,

            MockRegister::V0 => Vector128,
            MockRegister::V1 => Vector128,
            MockRegister::V2 => Vector128,
            MockRegister::V3 => Vector128,
            MockRegister::V4 => Vector128,
        }
    }

    fn extend(&self) -> Self {
        *self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub required_stack_alignment: u32,
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
            required_stack_alignment: 0,
        }
    }
}

impl CallingConventionInfo<MockRegister> for MockFunctionAttribute {
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
        self.return_reg
    }

    fn reserved_stack_space(&self) -> u32 {
        self.reserved_stack
    }

    fn callee_saved_registers(&self) -> &[MockRegister] {
        self.callee_saved.as_slice()
    }

    fn always_saved_registers(&self) -> &[MockRegister] {
        self.always_saved.as_slice()
    }

    fn stack_cleanup_behaviour(&self) -> StackCleanup {
        self.stack_cleanup
    }

    fn stack_parameter_order(&self) -> StackParameterOrder {
        self.stack_param_order
    }

    fn required_stack_alignment(&self) -> u32 {
        self.required_stack_alignment
    }

    fn scratch_registers(&self) -> &[MockRegister] {
        &[]
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
        required_stack_alignment: 1
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
        required_stack_alignment: 1
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
        required_stack_alignment: 1
    };

    /// A calling convention that is similar to x86 Microsoft 'fastcall', but for our pretend architecture.
    pub static ref FASTCALL_LIKE_FUNCTION_ATTRIBUTE: MockFunctionAttribute = MockFunctionAttribute {
        int_params: vec![ MockRegister::R1, MockRegister::R2 ], // first 2 params on stack
        float_params: vec![],
        vector_params: vec![],
        return_reg: MockRegister::R1,
        reserved_stack: 0,
        callee_saved: vec![MockRegister::R3, MockRegister::R4],
        always_saved: vec![],
        stack_cleanup: StackCleanup::Callee,  // callee cleanup
        stack_param_order: StackParameterOrder::RightToLeft,
        required_stack_alignment: 1
    };

    /// A calling convention that is similar to Microsoft 'x64', but for our pretend architecture.
    pub static ref MICROSOFTX64_LIKE_FUNCTION_ATTRIBUTE: MockFunctionAttribute = MockFunctionAttribute {
        int_params: vec![ MockRegister::R1, MockRegister::R2 ], // first 2 params on stack
        float_params: vec![],
        vector_params: vec![],
        return_reg: MockRegister::R1,
        reserved_stack: 32,
        callee_saved: vec![MockRegister::R3, MockRegister::R4],
        always_saved: vec![],
        stack_cleanup: StackCleanup::Callee,  // callee cleanup
        stack_param_order: StackParameterOrder::RightToLeft,
        required_stack_alignment: 16
    };
}
