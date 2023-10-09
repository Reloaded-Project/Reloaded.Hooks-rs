# Operations

!!! info "This page provides a reference for all of the various 'operations' implemented by individual JIT(s)."

!!! tip "For more information about each of the operations, see the source code 😉 (`enum Operation<T>`)."

## Needed for Basic Hooking Support

### JumpRelative

!!! info "Represents jumping to a relative offset from current instruction pointer."

=== "Rust"

    ```rust
    let jump_rel = JumpRelativeOperation {
        target_address: 0x200,
    };
    ```

=== "x64 (+- 2GiB)"

    ```asm
    jmp 0x200 ; Jump to address at current IP + 0x200
    ```

=== "ARM64 (+- 128MiB)"

    ```asm
    b 0x200 ; Branch to address at current IP + 0x200
    ```

=== "x86 (+- 2GiB)"

    ```asm
    jmp 0x200 ; Jump to address at current IP + 0x200
    ```

### JumpAbsolute

!!! info "Represents jumping to an absolute address stored in a register."

!!! note "JIT is free to encode this as a relative branch if it's possible."

=== "Rust"

    ```rust
    let jump_abs = JumpAbsoluteOperation {
        scratch_register: rax,
        target_address: 0x123456,
    };
    ```

=== "x64"

    ```asm
    mov rax, 0x123456 ; Move target address into rax
    jmp rax ; Jump to address in rax
    ```

=== "ARM64"

    ```asm
    MOVZ x9, #0x3456        ; Set lower bits.
    MOVK x9, #0x12, LSL #16 ; Move upper bits
    br x9                   ; Branch to location
    ```

=== "x86"

    ```asm
    mov eax, 0x123456 ; Move target address into eax
    jmp eax ; Jump to address in eax
    ```

!!! note "We prefer this approach to `absolute jump` because it is faster performance wise."

### JumpAbsoluteIndirect

!!! info "Represents jumping to an absolute address stored in a memory address."

=== "Rust"

    ```rust
    let jump_ind = JumpIndirectOperation {
        target_address: 0x123456,
    };
    ```

=== "x64 (< 2GiB)"

    ```asm
    jmp qword [0x123456] ; Jump to address stored at 0x123456
    ```

=== "ARM64 (< 256MiB)"

    ```asm
    MOVZ x9, #0x123, LSL #16 ; Move 0x123000
    LDR  x9, [x9, #0x456]    ; Load the value from offset 0x456 into x1
    br x9                    ; Branch to location
    ```

=== "x86 (< 2GiB)"

    ```asm
    jmp dword [0x123456] ; Jump to address stored at 0x123456
    ```

* Values in brackets indicate max address usable.

!!! warning "On MacOS, this is not usable, because memory < 2GiB is restricted from access."

## Needed for Wrapper Generation

!!! info "This includes functionality like 'parameter injection'."

### Mov

!!! info "Represents a move operation between two registers."

=== "Rust"

    ```rust
    let move_op = MovOperation {
        source: r8,
        target: r9,  
    };
    ```

=== "x64"

    ```asm
    mov r9, r8 ; Move r8 into r9
    ```

=== "ARM64"

    ```asm
    mov x9, x8 ; Move x8 into x9
    ```
    
=== "x86"

    ```asm
    mov ebx, eax ; Move eax into ebx
    ```

### MovFromStack

!!! info "Represents a move operation from the stack into a register."

=== "Rust"

    ```rust
    let move_from_stack = MovFromStackOperation {
        stack_offset: 8,
        target: rbx,
    };
    ```

=== "x64"

    ```asm
    mov rbx, [rsp + 8] ; Move value at rsp + 8 into rbx
    ```

=== "ARM64"

    ```asm
    ldr x9, [sp, #8] ; Load value at sp + 8 into x9
    ```

=== "x86"

    ```asm
    mov ebx, [esp + 8] ; Move value at esp + 8 into ebx
    ```

### Push

!!! info "Represents pushing a register onto the stack."

=== "Rust"

    ```rust
    let push = PushOperation {
        register: r9,
    };
    ```

=== "x64"

    ```asm
    push r9 ; Push rbx onto the stack
    ```

=== "ARM64"

    ```asm
    sub sp, sp, #8 ; Decrement stack pointer
    str x9, [sp] ; Store x9 on the stack
    ```

=== "x86"

    ```asm
    push ebx ; Push ebx onto the stack
    ```

### PushStack

!!! info "Represents pushing a value from the stack to the stack."

=== "Rust"

    ```rust
    let push_stack = PushStackOperation {
        offset: 8,
        item_size: 8,
    };
    ```

=== "x64"

    ```asm
    push qword [rsp + 8] ; Push value at rsp + 8 onto the stack
    ```

=== "ARM64"

    ```asm
    ldr x9, [sp, #8] ; Load value at sp + 8 into x9
    sub sp, sp, #8 ; Decrement stack pointer
    str x9, [sp] ; Push x9 onto the stack
    ```

=== "x86"

    ```asm
    push [esp + 8] ; Push value at esp + 8 onto the stack
    ```

### PushConstant

!!! info "Represents pushing a constant value onto the stack."

=== "Rust"

    ```rust
    let push_const = PushConstantOperation {
        value: 10,
    };
    ```

=== "x64"

    ```asm
    push 10 ; Push constant value 10 onto stack
    ```

=== "ARM64"

    ```asm
    sub sp, sp, #8 ; Decrement stack pointer
    mov x9, 10 ; Move constant 10 into x9
    str x9, [sp] ; Store x9 on the stack
    ```

=== "x86"

    ```asm
    push 10 ; Push constant value 10 onto stack
    ```

### StackAlloc

!!! info "Represents adjusting the stack pointer."

=== "Rust"

    ```rust
    let stack_alloc = StackAllocOperation {
        operand: 8,
    };
    ```

=== "x64"

    ```asm
    sub rsp, 8 ; Decrement rsp by 8
    ```

=== "ARM64" 

    ```asm
    sub sp, sp, #8 ; Decrement sp by 8
    ```

=== "x86"

    ```asm
    sub esp, 8 ; Decrement esp by 8
    ```

### Pop

!!! info "Represents popping a value from the stack into a register."

=== "Rust"

    ```rust
    let pop = PopOperation {
        register: rbx,
    };
    ```

=== "x64"

    ```asm
    pop rbx ; Pop value from stack into rbx
    ```

=== "ARM64"

    ```asm
    ldr x9, [sp] ; Load stack top into x9
    add sp, sp, #8 ; Increment stack pointer
    ```

=== "x86"

    ```asm
    pop ebx ; Pop value from stack into ebx
    ```

### XChg

!!! info "Represents exchanging the contents of two registers."

!!! note "On some architectures (e.g. ARM64) this requires a scratch register."

=== "Rust"

    ```rust
    let xchg = XChgOperation {
        register1: r9,
        register2: r8,
        scratch: None,
    };
    ```

=== "x64"

    ```asm
    xchg r8, r9 ; Swap r8 and r9
    ```

=== "ARM64"

    ```asm
    // ARM doesn't have xchg instruction
    mov x10, x8 ; Move x8 into x10 (scratch register)
    mov x8, x9 ; Move x9 into x8
    mov x9, x10 ; Move original x8 (in x10) into x9
    ```

=== "x86"

    ```asm
    xchg eax, ebx ; Swap eax and ebx
    ```

### CallAbsolute

!!! info "Represents calling an absolute address stored in a register or memory."

=== "Rust"

    ```rust
    let call_abs = CallAbsoluteOperation {
        scratch_register: r9,
        target_address: 0x123456,
    };
    ```

=== "x64"

    ```asm
    mov rax, 0x123456 ; Move target address into rax
    call r9 ; Call address in rax
    ```

=== "ARM64"

    ```asm
    adr x9, target_func ; Load address of target function into x9
    blr x9 ; Branch and link to address in x9
    ```

=== "x86"

    ```asm
    mov eax, 0x123456 ; Move target address into eax
    call eax ; Call address in eax
    ```

### CallRelative

!!! info "Represents calling a relative offset from current instruction pointer."

=== "Rust"

    ```rust
    let call_rel = CallRelativeOperation {
        target_address: 0x200,
    };
    ```

=== "x64"

    ```asm
    call 0x200 ; Call address at current IP + 0x200
    ```

=== "ARM64"

    ```asm
    bl 0x200 ; Branch with link to address at current IP + 0x200
    ```

=== "x86"

    ```asm
    call 0x200 ; Call address at current IP + 0x200
    ```

### Return

!!! info "Represents returning from a function call."

=== "Rust"

    ```rust
    let ret = ReturnOperation {
        offset: 4,
    };
    ```

=== "x64"

    ```asm
    ret ; Return
    ret 4 ; Return and add 4 to stack pointer
    ```

=== "ARM64"

    ```asm
    ret ; Return
    add sp, sp, #4 ; Add 4 to stack pointer
    ret ; Return
    ```

=== "x86"

    ```asm
    ret ; Return
    ret 4 ; Return and add 4 to stack pointer
    ```

## Architecture Specific Operations

!!! note "These operations are only available on certain architectures."

!!! note "These are non essential, but can improve compatibility/performance."

!!! tip "Enabled by setting `JitCapabilities::CanEncodeIPRelativeCall` and `JitCapabilities::CanEncodeIPRelativeJump` in JIT."

### CallIpRelative

!!! info "Represents calling an IP-relative offset where target address is stored."

=== "Rust"

    ```rust
    let call_rip_rel = CallIpRelativeOperation {
        target_address: 0x1000,
    };
    ```

=== "x64"

    ```asm
    call qword [rip - 16] ; Address 0x1000 is at RIP-16 and contains raw address to call
    ```

=== "ARM64 (+- 1MB)"

    ```asm
    ldr x9, 4 ; Read item in a multiple of 4 bytes relative to PC
    blr x9    ; Branch call to location
    ```

=== "ARM64 (+- 4GB)"

    ```asm
    adrp x9, [291]       ; Load 4K page, relative to PC. (round address down to 4096)
    ldr x9, [x9, 1110]   ; Read address from offset in 4K page.
    blr x9               ; Branch to location
    ```

### JumpIpRelative

!!! info "Represents jumping to an IP-relative offset where target address is stored."

=== "Rust"

    ```rust
    let jump_rip_rel = JumpIpRelativeOperation {
        target_address: 0x1000,
    };
    ```

=== "x64"

    ```asm
    jmp qword [rip - 16] ; Address 0x1000 is at RIP-16 and contains raw address to jump
    ```

=== "ARM64 (+- 1MB)"

    ```asm
    ldr x9, 4 ; Read item in a multiple of 4 bytes relative to PC
    br x9     ; Branch call to location
    ```

=== "ARM64 (+- 4GB)"

    ```asm
    adrp x9, [291]       ; Load 4K page, relative to PC. (round address down to 4096)
    ldr x9, [x9, 1110]   ; Read address from offset in 4K page.
    br x9                ; Branch call to location
    ```

## Optimized Push/Pop Operations

!!! tip "Enabled by setting `JitCapabilities::CanMultiPush` in JIT."

### MultiPush

!!! info "Represents pushing multiple registers onto the stack."

!!! note "Implementations must support push/pop of mixed registers (e.g. Reg+Vector)."

=== "Rust"

    ```rust  
    let multi_push = MultiPushOperation {
        registers: [
            PushOperation { register: rbx },
            PushOperation { register: rax },
            PushOperation { register: rcx },
            PushOperation { register: rdx },
        ],
    };
    ```

=== "x64"

    ```asm
    push rbx
    push rax
    push rcx
    push rdx ; Push rbx, rax, rcx, rdx onto the stack
    ```

=== "ARM64"

    ```asm
    sub sp, sp, #32 ; Decrement stack pointer by 32 bytes  
    stp x9, x8, [sp] ; Store x9 and x8 on the stack
    stp x11, x10, [sp, #16] ; Store x11 and x10 on the stack  
    ```

=== "x86"

    ```asm
    push ebx
    push eax
    push ecx
    push edx ; Push ebx, eax, ecx, edx onto the stack
    ```

### MultiPop

!!! info "Represents popping multiple registers from the stack."

!!! note "Implementations must support push/pop of mixed registers (e.g. Reg+Vector)."

=== "Rust" 

    ```rust
    let multi_pop = MultiPopOperation {
        registers: [
            PopOperation { register: rdx },
            PopOperation { register: rcx },
            PopOperation { register: rax },
            PopOperation { register: rbx },
        ],
    };
    ```

=== "x64"

    ```asm
    pop rdx
    pop rcx
    pop rax
    pop rbx ; Pop rdx, rcx, rax, rbx from the stack
    ```

=== "ARM64"

    ```asm
    ldp x11, x10, [sp], #16 ; Load x11 and x10 from stack and update stack pointer
    ldp x9, x8, [sp], #16 ; Load x9 and x8 from stack and update stack pointer
    ```

=== "x86"

    ```asm
    pop edx
    pop ecx
    pop eax
    pop ebx ; Pop edx, ecx, eax, ebx from the stack
    ```