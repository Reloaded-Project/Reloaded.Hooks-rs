use64

; Adds two 64-bit integers together, returning in RAX
; uses MSFT x64 convention.

; For testing branch hooks, we hook the 'add_wrapper' function
; Build with ./fasm -p 1 to avoid optimization

add_fn:
    mov rax, rcx ; Move num1 into RAX
    add rax, rdx ; Add num2 to RAX (RAX = RAX + RDX)
    ret

; Wrapper function calling 'add'
add_wrapper:
    ; Call the 'add' function
    ; Since add uses rcx and rdx, no need to move arguments
    sub rsp, 40h
    call add_fn
    add rsp, 40h
    ret

target_function:
    mov rax, rcx ; Move num1 into RAX
    add rax, rdx ; Add num2 to RAX (RAX = RAX + RDX)
    inc rax ; Add 1 for the test.
    ret