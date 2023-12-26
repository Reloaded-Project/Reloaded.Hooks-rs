use crate::x86::Register;
use crate::x86::Register::*;
use derive_more::Deref;
use derive_more::DerefMut;
use reloaded_hooks_portable::api::calling_convention_info::{
    GenericCallingConvention, StackCleanup, StackParameterOrder,
};

/// A variant of `GenericCallingConvention` for x86.
///
/// This struct is specialized for x86 and includes commonly used calling conventions such
/// as Cdecl, Stdcall, Fastcall, and others.
///
/// # Examples
///
/// ```rust
/// use reloaded_hooks_x86_sys::x86::calling_convention::CallingConvention;
/// let cdecl_convention = CallingConvention::cdecl();
/// ```
#[derive(Debug, Clone, PartialEq, DerefMut, Deref)]
pub struct CallingConvention<'a> {
    convention: GenericCallingConvention<'a, Register>,
}

static CDECL: CallingConvention = CallingConvention {
    convention: GenericCallingConvention::<Register> {
        int_parameters: &[],
        float_parameters: &[],
        vector_parameters: &[],
        return_register: eax,
        reserved_stack_space: 0,
        callee_saved_registers: &[ebx, esi, edi, ebp],
        always_saved_registers: &[],
        stack_cleanup: StackCleanup::Caller,
        stack_parameter_order: StackParameterOrder::RightToLeft,
        required_stack_alignment: 0,
    },
};

static STDCALL: CallingConvention = CallingConvention {
    convention: GenericCallingConvention::<Register> {
        int_parameters: &[],
        float_parameters: &[],
        vector_parameters: &[],
        return_register: eax,
        reserved_stack_space: 0,
        callee_saved_registers: &[ebx, esi, edi, ebp],
        always_saved_registers: &[],
        stack_cleanup: StackCleanup::Callee,
        stack_parameter_order: StackParameterOrder::RightToLeft,
        required_stack_alignment: 0,
    },
};

static FASTCALL: CallingConvention = CallingConvention {
    convention: GenericCallingConvention::<Register> {
        int_parameters: &[ecx, edx],
        float_parameters: &[],
        vector_parameters: &[],
        return_register: eax,
        reserved_stack_space: 0,
        callee_saved_registers: &[ebx, esi, edi, ebp],
        always_saved_registers: &[],
        stack_cleanup: StackCleanup::Caller,
        stack_parameter_order: StackParameterOrder::RightToLeft,
        required_stack_alignment: 0,
    },
};

static MICROSOFT_THISCALL: CallingConvention = CallingConvention {
    convention: GenericCallingConvention::<Register> {
        int_parameters: &[ecx],
        float_parameters: &[],
        vector_parameters: &[],
        return_register: eax,
        reserved_stack_space: 0,
        callee_saved_registers: &[ebx, esi, edi, ebp],
        always_saved_registers: &[],
        stack_cleanup: StackCleanup::Callee,
        stack_parameter_order: StackParameterOrder::RightToLeft,
        required_stack_alignment: 0,
    },
};

static CLRCALL: CallingConvention = CallingConvention {
    convention: GenericCallingConvention::<Register> {
        int_parameters: &[ecx, edx],
        float_parameters: &[],
        vector_parameters: &[],
        return_register: eax,
        reserved_stack_space: 0,
        callee_saved_registers: &[ebx, esi, edi, ebp],
        always_saved_registers: &[],
        stack_cleanup: StackCleanup::Callee,
        stack_parameter_order: StackParameterOrder::LeftToRight,
        required_stack_alignment: 0,
    },
};

impl<'a> CallingConvention<'a> {
    /// C declaration calling convention (Cdecl).
    /// - Parameters are passed on the stack right to left.
    /// - Floating-point parameters are typically on the stack.
    /// - Caller is responsible for stack cleanup.
    pub fn cdecl() -> &'a Self {
        &CDECL
    }

    /// Standard calling convention (Stdcall).
    /// - Parameters are passed on the stack right to left.
    /// - Floating-point parameters are typically on the stack.
    /// - Callee is responsible for stack cleanup.
    pub fn stdcall() -> &'a Self {
        &STDCALL
    }

    /// Fastcall calling convention.
    /// - First two integer parameters are passed in ECX and EDX registers.
    /// - Remaining parameters are passed on the stack right to left.
    /// - Floating-point parameters are typically on the stack.
    /// - Caller is responsible for stack cleanup.
    pub fn fastcall() -> &'a Self {
        &FASTCALL
    }

    /// Microsoft thiscall convention.
    /// - Used for C++ member functions.
    /// - 'this' pointer passed in ECX, other parameters on the stack right to left.
    /// - Callee is responsible for stack cleanup.
    pub fn microsoft_thiscall() -> &'a Self {
        &MICROSOFT_THISCALL
    }

    /// GCC thiscall convention.
    /// - Similar to Microsoft thiscall but used in GCC for C++ member functions.
    /// - 'this' pointer as the first stack parameter, other parameters on the stack right to left.
    /// - Caller is responsible for stack cleanup.
    pub fn gcc_thiscall() -> &'a Self {
        // Cdecl but 'this' pointer is the first parameter
        &CDECL
    }

    /// .NET runtime calling convention (ClrCall).
    /// - Arguments are pushed onto the stack left to right.
    /// - Callee is responsible for stack cleanup.
    pub fn clrcall() -> &'a Self {
        &CLRCALL
    }

    /// Returns a [`CallingConvention`] based on the provided [`PresetCallingConvention`].
    pub fn from_preset(convention_type: PresetCallingConvention) -> &'a Self {
        match convention_type {
            PresetCallingConvention::Cdecl => Self::cdecl(),
            PresetCallingConvention::Stdcall => Self::stdcall(),
            PresetCallingConvention::Fastcall => Self::fastcall(),
            PresetCallingConvention::MicrosoftThiscall => Self::microsoft_thiscall(),
            PresetCallingConvention::GCCThiscall => Self::gcc_thiscall(),
            PresetCallingConvention::ClrCall => Self::clrcall(),
        }
    }

    /// Returns the default calling convention for the current platform.
    /// This method uses conditional compilation to determine the appropriate
    /// convention based on the target architecture and operating system.
    pub fn default_for_current_platform() -> &'a Self {
        #[cfg(windows)]
        {
            Self::stdcall()
        }

        // TODO: Add other platforms here as needed.
        #[cfg(not(windows))]
        {
            Self::cdecl()
        }
    }
}

/// This enum provides information on various commonly seen calling conventions and how
/// to call functions utilizing them.
pub enum PresetCallingConvention {
    /// C declaration calling convention.
    /// - Integer parameters: Passed on stack right to left.
    /// - Vector parameters: Typically passed on stack; usage varies by compiler.
    /// - Additional parameters: Pushed onto stack right to left.
    /// - Return register: EAX (integer), ST0 (float, FPU stack).
    /// - Cleanup: Caller
    Cdecl,

    /// Standard calling convention.
    /// - Integer parameters: Passed on stack right to left.
    /// - Vector parameters: Typically passed on stack; usage varies by compiler.
    /// - Additional parameters: Pushed onto stack right to left.
    /// - Return register: EAX (integer), ST0 (float, FPU stack).
    /// - Cleanup: Callee
    Stdcall,

    /// Fast calling convention.
    /// - Integer parameters: ECX, EDX (first two, left to right), others on stack right to left.
    /// - Vector parameters: Passed in XMM0, XMM1 (first two, left to right), others on stack.
    /// - Additional parameters: Pushed onto stack right to left.
    /// - Return register: EAX (integer), XMM0 (float, vector).
    /// - Cleanup: Caller
    Fastcall,

    /// Microsoft thiscall convention.
    /// - Integer parameters: ECX (for 'this' pointer), others on stack right to left.
    /// - Vector parameters: ECX (for 'this' pointer in vectorized classes), others on stack.
    /// - Additional parameters: Pushed onto stack right to left.
    /// - Return register: EAX (integer), ST0 (float, FPU stack).
    /// - Cleanup: Callee
    MicrosoftThiscall,

    /// GCC thiscall convention.
    /// - Integer parameters: Passed on stack right to left, 'this' pointer as the first parameter.
    /// - Vector parameters: Passed on stack, 'this' pointer as the first parameter.
    /// - Additional parameters: Pushed onto stack right to left.
    /// - Return register: EAX (integer), ST0 (float, FPU stack).
    /// - Cleanup: Caller
    GCCThiscall,

    /// .NET runtime calling convention.
    /// - Integer parameters: ECX, EDX (first two, left to right), others on stack left to right.
    /// - Vector parameters: Passed in XMM0, XMM1 (first two, left to right), others on stack.
    /// - Additional parameters: Pushed onto stack left to right.
    /// - Return register: EAX (integer), XMM0 (float, vector).
    /// - Cleanup: Callee
    ClrCall,
    /*
    /// User-defined calling convention (Hex-Rays, IDA).
    /// - Integer parameters: Depends on function.
    /// - Vector parameters: Depends on function.
    /// - Additional parameters: Depends on function.
    /// - Return register: Varies based on function.
    /// - Cleanup: Caller
    // Usercall,

    /// User-defined calling convention (Hex-Rays, IDA), callee cleanup.
    /// - Integer parameters: Depends on function.
    /// - Vector parameters: Depends on function.
    /// - Additional parameters: Depends on function.
    /// - Return register: Varies based on function.
    /// - Cleanup: Callee
    // Userpurge,
    */
}
