# ARM64

!!! note "This is just a quick reference sheet for developers."

!!! info "ARM64 is not currently implemented."

- Code Alignment: 4 bytes

## Registers

| Register    | ARM64 (System V)                                                           | Volatile/Non-Volatile |
| ----------- | -------------------------------------------------------------------------- | --------------------- |
| `x0`-`x7`   | Parameter/Result Registers                                                 | Volatile              |
| `x8`        | Indirect result location register                                          | Volatile              |
| `x9`-`x15`  | Local Variables                                                            | Volatile              |
| `x16`-`x17` | Intra-procedure-call scratch registers                                     | Volatile              |
| `x18`       | Platform register, conventionally the TLS base                             | Volatile              |
| `x19`-`x28` | Registers saved across function calls                                      | Non-Volatile          |
| `x29`       | Frame pointer                                                              | Non-Volatile          |
| `x30`       | Link register                                                              | Volatile              |
| `sp`        | Stack pointer                                                              | Non-Volatile          |
| `xzr`       | Zero register, always reads as zero                                        | N/A                   |
| `x31`       | Stack pointer or zero register, contextually reads as either `sp` or `xzr` | N/A                   |

For floating point / SIMD registers:

| Register    | ARM64 (System V)                      | Volatile/Non-Volatile |
| ----------- | ------------------------------------- | --------------------- |
| `v0`-`v7`   | Parameter/Result registers            | Volatile              |
| `v8`-`v15`  | Temporary registers                   | Volatile              |
| `v16`-`v31` | Registers saved across function calls | Non-Volatile          |

## Calling Convention Inference

!!! note "It is recommended library users manually specify conventions in their hook functions.""

When the calling convention of `<your function>` is not specified, wrapper libraries must insert
the appropriate default convention in their wrappers.

### Rust

- `aarch64-unknown-linux-gnu`: SystemV
- `aarch64-pc-windows-msvc`: Windows ARM64

### C#

- `Linux ARM64`: SystemV
- `Windows ARM64`: Windows ARM64
