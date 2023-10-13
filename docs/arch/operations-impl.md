# Operations

!!! info "This page tells you which [Operations](./operations.md) are currently implemented for each architecture."

- ❌ Means it is not implemented.
- ✅ Means it is implemented.
- ❓ Means 'not applicable'.

## Needed for Basic Hooking Support

### [JumpRelative](./operations.md#jumprelative)

| Architecture      | Supported | Notes                                      |
| ----------------- | --------- | ------------------------------------------ |
| x64               | ✅         | +-2GiB                                     |
| x86               | ✅         | +-2GiB                                     |
| ARM64 (+- 128MiB) | ✅         | +-128MiB                                   |
| ARM64 (+- 4GiB)   | ✅         | Uses 3 instructions. Used if within range. |

### [JumpAbsolute](./operations.md#jumpabsolute)

| Architecture | Supported | Notes                                 |
| ------------ | --------- | ------------------------------------- |
| x64          | ✅         | Uses scratch register for efficiency. |
| x86          | ✅         | Uses scratch register for efficiency. |
| ARM64        | ✅         | Uses scratch register (required)      |

### [JumpAbsoluteIndirect](./operations.md#jumpabsoluteindirect)

| Architecture | Supported | Notes                                                                     |
| ------------ | --------- | ------------------------------------------------------------------------- |
| x86          | ❌         | Not yet implemented                                                       |
| x86          | ❌         | Not yet implemented                                                       |
| ARM64        | ❌         | Variant 0.                                                                |
| ARM64        | ✅         | Variant 1. Replaced with [JumpAbsolute](#jumpabsolute), for perf reasons. |

## Needed for Wrapper Generation

### [Mov](./operations.md#mov)  

| Architecture | Register to Register | Vector to Vector |
| ------------ | -------------------- | ---------------- |
| x64          | ✅                    | ❌                |
| x86          | ✅                    | ❌                |
| ARM64        | ✅                    | ✅                |

### [MovFromStack](./operations.md#movfromstack)

| Architecture | to Register | to Vector |
| ------------ | ----------- | --------- |
| x64          | ✅           | ❌         |
| x86          | ✅           | ❌         |
| ARM64        | ✅           | ✅         |

### [Push](./operations.md#push)

| Architecture | Register | Vector |
| ------------ | -------- | ------ |
| x64          | ✅        | ✅      |
| x86          | ✅        | ✅      |
| ARM64        | ✅        | ✅      |

### [PushStack](./operations.md#pushstack)

| Architecture | Supported | Notes                                     |
| ------------ | --------- | ----------------------------------------- |
| x64          | ✅         |                                           |
| x86          | ✅         |                                           |
| ARM64        | ✅         | Will use vector registers when available. |

### [PushConstant](./operations.md#pushconstant)

| Architecture | Supported | Notes                                           |
| ------------ | --------- | ----------------------------------------------- |
| x64          | ✅         |                                                 |
| x86          | ✅         |                                                 |
| ARM64        | ✅         | 2-5 instructions, depending on constant length. |

### [StackAlloc](./operations.md#stackalloc)

| Architecture | Supported |
| ------------ | --------- |
| x64          | ✅         |
| x86          | ✅         |
| ARM64        | ✅         |

### [Pop](./operations.md#pop)

| Architecture | to Register | to Vector | Notes |
| ------------ | ----------- | --------- | ----- |
| x64          | ✅           | ✅         |       |
| x86          | ✅           | ✅         |       |
| ARM64        | ✅           | ✅         |       |

### [XChg](./operations.md#xchg)

| Architecture | Registers | Vectors | Notes                      |
| ------------ | --------- | ------- | -------------------------- |
| x64          | ✅         | ✅ *     | *Requires scratch register |
| x86          | ✅         | ✅ *     | *Requires scratch register |
| ARM64        | ✅ *       | ✅ *     | *Requires scratch register |

### [CallAbsolute](./operations.md#callabsolute)

| Architecture     | Supported | Notes                                 |
| ---------------- | --------- | ------------------------------------- |
| x64 (register)   | ✅         | Uses scratch register for efficiency. |
| x86 (register)   | ✅         | Uses scratch register for efficiency. |
| ARM64 (register) | ✅         | Uses scratch register (required)      |

### [CallRelative](./operations.md#callrelative)

| Architecture | Supported | Notes    |
| ------------ | --------- | -------- |
| x64          | ✅         | +-2GiB   |
| x86          | ✅         | +-2GiB   |
| ARM64        | ✅         | +-128MiB |

### [Return](./operations.md#return)

| Architecture | Supported | Notes                         |
| ------------ | --------- | ----------------------------- |
| x64          | ✅         |                               |
| x86          | ✅         |                               |
| ARM64        | ✅         | 2 instructions if offset > 0. |

## Architecture Specific Operations

### [CallIpRelative](./operations.md#calliprelative)

| Architecture    | Supported | Notes           |
| --------------- | --------- | --------------- |
| x64             | ✅         |                 |
| x86             | ❓         | Unsupported.    |
| ARM64 (+- 1MiB) | ✅         | 2 instructions. |
| ARM64 (+- 4GiB) | ✅         | 3 instructions. |

### [JumpIpRelative](./operations.md#jumpiprelative)

| Architecture    | Supported | Notes           |
| --------------- | --------- | --------------- |
| x64             | ✅         |                 |
| x86             | ❓         | Unsupported.    |
| ARM64 (+- 1MiB) | ✅         | 2 instructions. |
| ARM64 (+- 4GiB) | ✅         | 3 instructions. |

## Optimized Push/Pop Operations

### [MultiPush](./operations.md#multipush)

| Architecture | Supported | Notes                                                        |
| ------------ | --------- | ------------------------------------------------------------ |
| x64          | ✅         |                                                              |
| x86          | ✅         |                                                              |
| ARM64        | ✅         | Might fall back to single pop/push if mixing register sizes. |

### [MultiPop](./operations.md#multipop)

| Architecture | Supported | Notes                                                        |
| ------------ | --------- | ------------------------------------------------------------ |
| x64          | ✅         |                                                              |
| x86          | ✅         |                                                              |
| ARM64        | ✅         | Might fall back to single pop/push if mixing register sizes. |