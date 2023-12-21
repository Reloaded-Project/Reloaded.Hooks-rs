; Adds two 32-bit integers together, returning in EAX
; uses cdecl calling convention.
use32
push ebp        ; Save old base pointer
mov ebp, esp    ; Set new base pointer
mov eax, [ebp+8] ; Move first argument (num1) into EAX
add eax, [ebp+12] ; Add second argument (num2) to EAX (EAX = EAX + num2)
pop ebp         ; Restore old base pointer
ret             ; Return to caller, who cleans the stack
