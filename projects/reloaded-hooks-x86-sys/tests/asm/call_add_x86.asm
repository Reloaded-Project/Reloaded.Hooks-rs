use32

; Adds two 32-bit integers together, returning in EAX
; uses cdecl calling convention.

; For testing branch hooks, we hook the 'add_wrapper' function
; Build with ./fasm -p 1 to avoid optimization

add_fn:
    push ebp        ; Save old base pointer
    mov ebp, esp    ; Set new base pointer
    mov eax, [ebp+8] ; Move first argument (num1) into EAX
    add eax, [ebp+12] ; Add second argument (num2) to EAX (EAX = EAX + num2)
    pop ebp         ; Restore old base pointer
    ret

; Wrapper function calling 'add'
add_wrapper:
    ; Prepare arguments for 'add' function
    ; Assuming 'add_wrapper' receives its parameters in the same manner
    push dword [esp+8] ; Push second argument (num2)
    push dword [esp+8] ; Push first argument (num1)
    call add_fn
    add esp, 8 ; Clean up the stack (remove parameters)
    ret

target_function:
    push ebp
    mov ebp, esp
    mov eax, [ebp+8]
    add eax, [ebp+12]
    inc eax ; Add 1 for the test.

    pop ebp
    ret
