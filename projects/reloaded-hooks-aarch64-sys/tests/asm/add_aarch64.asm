; Adds two 64-bit integers together, returning in X0
; uses AArch64 calling convention.

; Add num2 (in X1) to X0 (X0 = X0 + X1)
add x0, x0, x1
; Nop slide in case
nop
nop
nop
nop
ret