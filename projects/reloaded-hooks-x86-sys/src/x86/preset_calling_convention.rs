/// This enum provides information on various commonly seen calling conventions and how
/// to call functions utilizing them.
pub enum PresetCallingConvention {
    /// Parameters are passed right to left onto the function pushing onto the stack.
    /// Calling function pops its own arguments from the stack.
    /// (The calling function must manually restore the stack to previous state)
    ///
    /// - TargetRegisters:    N/A
    /// - ReturnRegister:     EAX    
    /// - Cleanup:            Caller
    Cdecl,

    /// Parameters are passed right to left onto the function pushing onto the stack.
    /// Called function pops its own arguments from the stack.
    ///
    /// - TargetRegisters:    N/A
    /// - ReturnRegister:     EAX    
    /// - Cleanup:            Callee
    Stdcall,

    /// The first two arguments are passed in from left to right into ECX and EDX.
    /// The others are passed in right to left onto stack.
    ///
    /// - TargetRegisters:    ECX, EDX
    /// - ReturnRegister:     EAX    
    /// - Cleanup:            Caller
    Fastcall,

    /// Variant of Stdcall where the pointer of the `this` object is passed into ECX and
    /// rest of the parameters passed as usual. The Callee cleans the stack.
    ///
    /// You should define your delegates with the (this) object pointer (IntPtr) as first parameter from the left.
    ///
    /// - TargetRegisters:    ECX
    /// - ReturnRegister:     EAX
    /// - Cleanup:            Callee
    ///
    /// For GCC variant of Thiscall, use Cdecl.
    MicrosoftThiscall,

    /// A variant of CDECL whereby the first parameter is the pointer to the `this` object.
    /// Everything is otherwise the same.
    ///
    /// - TargetRegisters:    N/A
    /// - ReturnRegister:     EAX    
    /// - Cleanup:            Caller
    GCCThiscall,

    /// A name given to custom calling conventions by Hex-Rays (IDA) that are cleaned up by the caller.
    /// You should declare the `FunctionAttribute` manually yourself.
    ///
    /// - TargetRegisters:    Depends on Function
    /// - ReturnRegister:     Depends on Function
    /// - Cleanup:            Caller
    Usercall,

    /// A name given to custom calling conventions by Hex-Rays (IDA) that are cleaned up by the callee.
    /// You should declare the `FunctionAttribute` manually yourself.
    ///
    /// - TargetRegisters:    Depends on Function
    /// - ReturnRegister:     Depends on Function
    /// - Cleanup:            Callee
    Userpurge,

    /// The calling convention internally used by the .NET runtime.
    /// Arguments are pushed to the stack LEFT TO RIGHT unlike other conventions,
    /// so please reverse the order of all parameters past the second one.
    ///
    /// - TargetRegisters: ECX, EDX
    /// - ReturnRegister:  EAX    
    /// - Cleanup:         Callee
    ClrCall,
}
