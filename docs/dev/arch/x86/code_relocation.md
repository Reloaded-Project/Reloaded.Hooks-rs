# Code Relocation

!!! info "This page provides a listing of all instructions rewritten as part of the [Code Relocation](../overview.md#code-relocation) process for x86 architecture."

This page provides a comprehensive overview of the instruction rewriting techniques used in the code 
relocation process, specifically tailored for the x64 architecture.

## Any Instruction within 2GiB Range

!!! tip "If the new relative branch target is within the encodable range, it is left as relative."

### Example: Within Relative Range

**Original**: (`EB 02`)  
- `jmp +2`  

**Relocated**: (`E9 FF 0F 00 00`)  
- `jmp +4098`  

```rust
// Parameters for test case:
// - Original Code (Hex)
// - Original Address
// - New Address
// - New Expected Code (Hex)
`#[case::simple_branch("eb02", 4096, 0, "e9ff0f0000")]
```

!!! note "In x86, any address is reachable from any address"

    This is due to integer over/underflow and immediates being 2GiB in size. Therefore relocation
    simply involves extending the immediate as needed, i.e. `jmp 0x12` to `jmp 0x123012` etc.

    The rest of the page will therefore leave out relative cases, and only focus on offsets greater
    than 2GiB.

## x64 Rewriter: Going Beyond the 2GiB Offset

!!! warning "The x64 rewriter is only suitable for rewriting function prologues."

To be able to perform a lot of actions in a position independent manner, this rewriter uses a dummy
'scratch' register which it will overwrite. 

Scratch register is determined by the following logic:  

- Start with `Caller Saved Registers` (these restored after function call).  
- Remove all registers used in code being rewritten.  

Because rewriting a lot of code will lead to register exhaustion, it must be reiterated the rewriter can only be used for small bits of code.

!!! danger "x64 has over 5000 ‼️ instructions that require rewriting. Only a couple hundred are tested currently"

## Relative Branches

Instructions such as `JMP`, `CALL`, etc.

**Behaviour**:

If out of range, it is rewritten using a combination of `MOV` (move the absolute address into a register) followed by `JMP` or `CALL` to that register.

### Example

**Original**: (`EB 02`)  
- `jmp +2`  

**Relocated**: (`48 B8 04 00 00 80 00 00 00 00 FF E0`)  
- `mov rax, 0x80000004`  
- `jmp rax`  

```rust
// Parameters for test case:
// - Original Code (Hex)
// - Original Address
// - New Address
// - New Expected Code (Hex)
#[case::to_abs_jmp_i8("eb02", 0x80000000, 0, "48b80400008000000000ffe0")]
```

## Jump Conditional

Instructions such as `jne`, `jg` etc.

**Behaviour**:  

- Inverts the branch condition, then jumps over an absolute jump that is encoded using a `MOV` to set the address and a `JMP` to that address.

### Example

**Example**:

**Original**: (`70 02`)  
- `jo +2`  

**Relocated**: (`71 0C 48 B8 04 00 00 80 00 00 00 FF E0`):  
- `jno +12 <skip>`  
- `mov rax, 0x80000004`  
- `jmp rax`  

```rust
// Parameters for test case:
// - Original Code (Hex)
// - Original Address
// - New Address
// - New Expected Code (Hex)
#[case::jo("7002", 0x80000000, 0, "710c48b80400008000000000ffe0")]
```

## Loop Instructions

Instructions such as `LOOP`, `LOOPE`, and `LOOPNE`.

**Behaviour**:  

Handled by either:  

- Manually decrementing `ECX` and using a conditional jump based on the zero flag. (i.e. extend 'loop' address to 32-bit)  

or 

- Branching the `loop` function in the opposite direction.  

The strategy used depends on the original instruction.

### Example: Branch in Opposite Direction

**Original**: (`E2 FA`)  
- `loop -3`  

**Relocated**: (`E2 02 EB 0C 48 B8 FD 0F 00 80 00 00 00 00 FF E0`)  
- `loop +2`  
- `jmp 0x11`  
- `movabs rax, 0x80000ffd`
- `jmp rax`  

```rust
// Parameters for test case:
// - Original Code (Hex)
// - Original Address
// - New Address
// - New Expected Code (Hex)
#[case::loop_backward_abs("50e2fa", 0x80001000, 0, "50e202eb0c48b8fd0f008000000000ffe0")]
```

## JCX Instructions
 
Instructions such as `JCXZ`, `JECXZ`, `JRCXZ`.

**Behaviour**:  

- If the target is within 32-bit range, it uses an optimized `IMM32` encoding.  
- If out of 32-bit range, it uses a `TEST` instruction followed by a conditional jump.  

### Example

**Original**: (`E3 FA`)  
- `jrcxz -3`  

**Relocated**: (`E3 02 EB 0C  48 B8 FD 0F 00 80 00 00 00 00 FF E0`)  
- `jrcxz +5`  
- `jmp 0x11`  
- `mov rax, 0x80000ffd`  
- `jmp rax`  

```rust
// Parameters for test case:
// - Original Code (Hex)
// - Original Address
// - New Address
// - New Expected Code (Hex)
#[case::jrcxz_abs("e3fa", 0x80001000, 0, "e302eb0c48b8fd0f008000000000ffe0")]
```

## RIP Relative Operand

!!! warning "At time of writing, this covers around 2800 ‼️ instructions"

!!! danger "Only around a 100 are covered by unit tests though."

Covers all instructions which have an IP relative operand, i.e. read/write to a memory address
which is relative to the address of the next instruction.

**Behaviour**:  

Replace RIP relative operand with a scratch register with the originally intended memory address.

### Example

**Original**: (`48 8B 1D 08 00 00 00`)  
- `mov rbx, [rip + 8]`  

**Relocated**: (`48 B8 0F 00 00 00 01 00 00 00 48 8B 18`)  
- `mov rax, 0x10000000f`  
- `mov rbx, [rax]`  

```rust
// Parameters for test case:
// - Original Code (Hex)
// - Original Address
// - New Address
// - New Expected Code (Hex)
#[case::mov_rhs("488b1d08000000", 0x100000000, 0, "48b80f00000001000000488b18")]
```

### How this is Done

`reloaded-hooks-rs` uses the [iced](https://github.com/icedland/iced) library under the hood for
assembly and disassembly. 

In iced, operands can be broken down to 3 main types:  

| Name     | Note                       |
| -------- | -------------------------- |
| register | Including Vector Registers |
| memory   | i.e. `[rax]` or `[rip + 4]`|
| imm      | Immediate, 8/16/32/64      |

!!! note "Immediates use multiple types, e.g. `Immediate8`, `Immediate16` etc. but on assembler side you can pass them all as Immediate32, so you can group them."

Each instruction can have 0-5 operands, where there is at max 1 operand which can be RIP relative.

To handle this, a script `projects/code-generators/x86/generate_enum_ins_combos.py` was used to dump
all possible operand permutations from `Iced` source. Then I wrote functions to handle each possible permutation.

**1 Operand**:  

- rip  

**2 Operands**:

- rip, imm
- rip, reg
- reg, rip

**3 Operands**:

- reg, reg, rip
- reg, rip, imm
- rip, reg, imm
- rip, reg, reg
- reg, rip, reg

**4 Operands**:

- reg, reg, rip, imm
- reg, reg, reg, rip

**5 Operands**:

- reg, reg, reg, rip, imm
- reg, reg, rip, reg, imm

If `reloaded-hooks-rs` encounters an instruction with RIP relative operand that uses any of the 
following operand permutations, it should successfully patch it.