# Interoperability (x86)

!!! note "Please read the [general](./hooking-strategy.md) section first, this contains x86 specific stuff."

#### Fallback Strategy: Free Space from Function Alignment

!!! info "See [General Section Notes](./hooking-strategy.md#free-space-from-function-alignment)."

- x86 programs align instructions on 16 byte boundaries. 
- Bytes `0x90` (GCC) or `0xCC` (MSVC) are commonly used for padding.

#### Fallback Strategy: Return Address Patching

!!! info "See [General Section Notes](./hooking-strategy.md#return-address-patching)."

We use x86 in the example for general section above.