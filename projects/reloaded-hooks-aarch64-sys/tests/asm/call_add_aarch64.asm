; ARM64 Assembly (AArch64)

; Adds two 64-bit integers together, returning in X0
; uses AArch64 calling convention.

add_fn:
    ; X0 already contains the first argument (num1)
    ; X1 contains the second argument (num2)
    add x0, x0, x1 
    ret                       ; Return, result is in X0

; Wrapper function calling 'add_fn'
add_wrapper:
    stp x29, x30, [sp, #-16]! ; Save frame pointer and link register, adjust stack
    mov x29, sp               ; Set frame pointer

    bl add_fn                 ; Call 'add_fn'

    ldp x29, x30, [sp], #16   ; Restore frame pointer and link register, adjust stack
    ret                       ; Return, result from 'add_fn' is in X0

target_function:
    ; X0 already contains the first argument (num1)
    ; X1 contains the second argument (num2)
    add x0, x0, x1
    add x0, x0, 1             ; Increment 1
    ret                       ; Return, result is in X0
