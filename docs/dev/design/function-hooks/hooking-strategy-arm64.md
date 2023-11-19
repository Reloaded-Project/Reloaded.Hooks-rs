# Interoperability (ARM64)

!!! note "Please read the [general](./hooking-strategy.md) section first, this contains ARM64 specific stuff."

## Fallback Strategy: Free Space from Function Alignment

!!! info "See [General Section Notes](./hooking-strategy.md#fallback-strategy-free-space-from-function-alignment)."

In the case of ARM64, padding is usually down with the following sequences:  
- `nop` (`0xD503201F`, big endian), used by GCC.  
- `and x0, x0` (`0x00000000`), used by MSVC.  

!!! note "Getting sufficient bytes to make good use of them in ARM64 is more uncommon than x86."