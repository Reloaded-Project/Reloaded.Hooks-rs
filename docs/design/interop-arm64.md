# Interoperability (ARM64)

!!! note "This page just contains some small notes and tidbits regarding interoperability."

On ARM64 there is no 'standard' way to hook functions. However, we will take inspiration from
[detours][detours] and use the following sequence when possible (12 bytes):  

```asm
adrp x17, [jmpval]      # Load 4K page, relative to PC.
ldr x17, [x17, jmpval]  # Offset 4K page
br x17                  # Branch to location
```

In the case this is not possible (no buffer in -+4GB range), we use [available free space from function alignment](#fallback-strategy-free-space-from-function-alignment)
if/when possible:

```asm
ADR x0, [offset]
br x0
```

Otherwise, we default to a 20 byte branch:

```asm
movz x0, 0x1111, lsl 48
movk x0, 0x2222, lsl 32
movk x0, 0x3333, lsl 16
movk x0, 0x4444
br x0
```

[detours]: https://github.com/microsoft/Detours/blob/4b8c659f549b0ab21cf649377c7a84eb708f5e68/src/detours.cpp#L981


## Fallback Strategy: Free Space from Function Alignment

!!! info "x86 processors typically fetch instructions 16 byte boundaries."

!!! info "To optimise for this, compilers pad the space between end of last function and start of next."

!!! tip "We can exploit this for code space 😉"

In the case of ARM64, padding is usually down with the following sequences:  
- `nop` (`0xD503201F`, big endian), used by GCC.  
- `and x0, x0` (`0x00000000`), used by MSVC.  

!!! note "Getting sufficient bytes to make good use of them in ARM64 is more uncommon than x86."