; Adds two 64-bit integers together, returning in RAX
; uses MSFT x64 convention.
use64
mov rax, rcx ; Move num1 into RAX
add rax, rdx ; Add num2 to RAX (RAX = RAX + RDX)

; Nop slide in case 
nop
nop
nop
nop
nop
nop
nop
ret