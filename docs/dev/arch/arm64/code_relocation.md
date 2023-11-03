# Code Relocation

!!! info "This page provides a listing of all instructions rewritten as part of the [Code Relocation](../overview.md#code-relocation) process."

## ADR(P)

**Purpose**:  

The `ADR` instruction in ARM architectures computes the address of a label and writes it to the destination register. 

**Behaviour**:  

The ADR(P) instruction is rewritten as one of the following:  
- ADR(P)  
- ADR(P) + ADD  
- MOV (1-4 instructions)  

**Example**:

1. **From ADRP to ADR**:
    ```rust
    // Before: ADRP x0, 0x101000
    // After: ADR x0, 0xFFFFF
    // Parameters: (old_instruction, old_address, new_address)
    rewrite_adr(0x000800B0_u32.to_be(), 0, 4097);
    ```

2. **Within 4GiB Range with Offset**:
    ```rust
    // Before: ADRP x0, 0x101000
    // After: 
    //  - ADRP x0, 0x102000
    //  - ADD x0, x0, 1
    rewrite_adr(0x000800B0_u32.to_be(), 4097, 0);
    ```

3. **Within 4GiB Range without Offset**:
    ```rust
    // Before: ADRP x0, 0x101000
    // After: ADRP x0, 0x102000
    rewrite_adr(0x000800B0_u32.to_be(), 4096, 0);
    ```

4. **Out of Range**:
    ```rust
    // PC = 0x100000000

    // Before: ADRP, x0, 0x101000
    // After: MOV IMMEDIATE 0x100101000
    rewrite_adr(0x000800B0_u32.to_be(), 0x100000000, 0);
    ```

## Branch (Conditional)

**Purpose**:  
The `Bcc` instruction in ARM architectures performs a conditional branch based on specific condition flags. 

**Behaviour**:  
The Branch Conditional instruction is rewritten as:  
- BCC  
- BCC <skip> + [B]  
- BCC <skip> + [ADRP + ADD + BR]  
- BCC <skip> + [MOV to Register + Branch Register]  

`<skip>` means, invert the condition, and jump over the code inside [] brackets.

**Example**:

1. **Within 1MiB**:
    ```rust
    // Before: b.eq #4
    // After: b.eq #-4092
    // Parameters: (old_instruction, old_address, new_address, scratch_register)
    rewrite_bcc(0x20000054_u32.to_be(), 0, 4096, Some(17));
    ```

2. **Within 128MiB**:
    ```rust
    // Before: b.eq #0
    // After: 
    //   - b.ne #8 
    //   - b #-0x80000000
    rewrite_bcc(0x00000054_u32.to_be(), 0, 0x8000000 - 4, Some(17));
    ```

3. **Within 4GiB Range with Address Adjustment**:
    ```rust
    // Before: b.eq #512
    // After: 
    //   - b.ne #16 
    //   - adrp x17, #0x8000000
    //   - add x17, #512
    //   - br x17
    rewrite_bcc(0x00100054_u32.to_be(), 0x8000000, 0, Some(17));
    ```

4. **Within 4GiB Range without Offset**:
    ```rust
    // Before: b.eq #512
    // After: 
    //   - b.ne #12
    //   - adrp x17, #-0x8000000 
    //   - br x17
    rewrite_bcc(0x00100054_u32.to_be(), 0, 0x8000000, Some(17));
    ```

5. **Last Resort**:
    ```rust
    // Before: b.eq #0
    // After: 
    //   - b.ne #12
    //   - movz x17, #0 
    //   - br x17
    rewrite_bcc(0x00000054_u32.to_be(), 0, 0x100000000, Some(17));
    ```

## Branch

!!! note "Including Branch+Link (BL)."

**Purpose**:  
The `B` (or `BL` for Branch+Link) instruction in ARM architectures performs a direct branch (or branch with link) to a specified address. When using the `BL` variant, the return address (the address of the instruction following the branch) is stored in the link register `LR`.

**Behaviour**:  
The Branch instruction is rewritten as one of the following:  
- B (or BL)  
- ADRP + BR  
- ADRP + ADD + BR  
- MOV <immediate> + BR  

**Example**:

1. **Direct Branch within Range**:
    ```rust
    // Before: b #4096
    // After: b #8192
    // Parameters: (old_instruction, old_address, new_address, scratch_register, link)
    rewrite_b(0x00040014_u32.to_be(), 8192, 4096, Some(17), false);
    ```

2. **Within 4GiB with Address Adjustment**:
    ```rust
    // Before: b #4096
    // After: 
    //   - adrp x17, #0x8000000
    //   - br x17
    rewrite_b(0x00040014_u32.to_be(), 0x8000000, 0, Some(17), false);
    ```

3. **Within 4GiB Range with Offset**:
    ```rust
    // Before: b #4096
    // After: 
    //   - adrp x17, #0x8000512
    //   - add x17, x17, #512
    //   - br x17
    rewrite_b(0x00040014_u32.to_be(), 0x8000512, 0, Some(17), false);
    ```

4. **Out of Range, Use MOV**:
    ```rust
    // Before: b #4096
    // After: 
    //   - movz x17, #... 
    //   - ...
    //   - br x17
    rewrite_b(0x00040014_u32.to_be(), 0x100000000, 0, Some(17), false);
    ```

5. **Branch with Link within Range**:
    ```rust
    // Before: bl #4096
    // After: bl #8192
    rewrite_b(0x00040094_u32.to_be(), 8192, 4096, Some(17), true);
    ```

## CBZ (Compare and Branch on Zero)

**Purpose**:  
The `CBZ` instruction in ARM architectures performs a conditional branch when the specified register is zero. If the register is not zero and the condition is not met, the next sequential instruction is executed.

**Behaviour**:  
The `CBZ` instruction is rewritten as one of the following:  
- CBZ  
- CBZ <skip> + [B]  
- CBZ <skip> + [ADRP + BR]  
- CBZ <skip> + [ADRP + ADD + BR]  
- CBZ <skip> + [MOV to Register + Branch Register]  

Here, `<skip>` is used to invert the condition and jump over the set of instructions inside the `[]` brackets if the condition is not met.

**Example**:

1. **Within 1MiB Range**:
    ```rust
    // Before: cbz x0, #4096
    // After: cbz x0, #8192
    // Parameters: (old_instruction, old_address, new_address)
    rewrite_cbz(0x008000B4_u32.to_be(), 8192, 4096, Some(17));
    ```

1. **Within 128MiB Range**:
    ```rust
    // Before: cbz x0, #4096
    // After: 
    //   - cbnz x0, #8
    //   - b #0x8000000
    rewrite_cbz(0x008000B4_u32.to_be(), 0x8000000, 4096, Some(17));
    ```

2. **Within 4GiB + 4096 aligned**:
    ```rust
    // Before: cbz x0, #4096
    // After: 
    //   - cbnz x0, <skip 3 instructions> 
    //   - adrp x17, #0x8000000
    //   - br x17
    rewrite_cbz(0x008000B4_u32.to_be(), 0x8000000, 0, Some(17));
    ```

3. **Within 4GiB with Offset**:
    ```rust
    // Before: cbz x0, #4096
    // After: 
    //   - cbnz x0, <skip 4 instructions>
    //   - adrp x17, #0x8000000
    //   - add x17, #512
    //   - br x17
    rewrite_cbz(0x008000B4_u32.to_be(), 0x8000512, 0, Some(17));
    ```

4. **Out of Range (Move and Branch)**:
    ```rust
    // Before: cbz x0, #4096
    // After: 
    //   - cbnz x0, <skip X instructions> 
    //   - mov x17, <immediate address>
    //   - br x17
    rewrite_cbz(0x008000B4_u32.to_be(), 0x100000000, 0, Some(17));
    ```

## LDR (Load Register)

!!! note "This includes Prefetch `PRFM` which shares opcode with LDR."

**Purpose**:  
The `LDR` instruction in ARM architectures is used to load a value from memory into a register. It can use various addressing modes, but commonly it involves an offset from a base register or the program counter.

**Behaviour**:  
The `LDR` instruction is rewritten as one of the following, depending on the relocation range:  

- LDR Literal
- ADRP + LDR (with Unsigned Offset)
- MOV Address to Register + LDR

The choice of rewriting strategy is based on the distance between the old address and the new one, with a preference for the most direct form that satisfies the required address range.

If the instruction is Prefetch `PRFM`, it is discarded if it can't be re-encoded as `PRFM (literal)`, as prefetching with multiple instructions is probably less efficient than not prefetching at all.

**Example**:

1. **Within 1MiB Range**:
    ```rust
    // Before: LDR x0, #0
    // After: LDR x0, #4096
    // Parameters: (opcode, new_imm12, rn)
    rewrite_ldr_literal(0x00000058_u32.to_be(), 4096, 0);
    ```

2. **Within 4GiB + 4096 aligned**:
    ```rust
    // Before: LDR x0, #0
    // After: 
    //   - adrp x0, #0x100000
    //   - ldr x0, [x0]
    // Parameters: (opcode, new_address, old_address)
    rewrite_ldr_literal(0x00000058_u32.to_be(), 0x100000, 0);
    ```

3. **Within 4GiB**:
    ```rust
    // Before: LDR x0, #512
    // After: 
    //   - adrp x0, #0x100000
    //   - ldr x0, [x0, #512]
    // Parameters: (opcode, new_address, old_address)
    rewrite_ldr_literal(0x00100058_u32.to_be(), 0x100000, 0);
    ```

4. **Out of Range (Last Resort)**:
    ```rust
    // Before: LDR x0, #512
    // After: 
    //   - movz x0, #0, lsl #16
    //   - movk x0, #0x1, lsl #32
    //   - ldr x0, [x0, #512]
    // Parameters: (opcode, new_address, old_address)
    rewrite_ldr_literal(0x00100058_u32.to_be(), 0x100000000, 0);
    ```

## TBZ (Test and Branch on Zero)

**Purpose**:  
The `TBZ` instruction in ARM architectures tests a specified bit in a register and performs a conditional branch if the bit is zero. If the tested bit is not zero, the next sequential instruction is executed.

**Behaviour**:  
The `TBZ` instruction is rewritten based on the distance to the new branch target. It is transformed into one of the following patterns:
- TBZ
- TBZ <skip> + B
- TBZ <skip> + ADRP + BR
- TBZ <skip> + ADRP + ADD + BR
- TBZ <skip> + MOV to Register + Branch Register

Here, `<skip>` is used to indicate a conditional skip over a set of instructions if the tested bit is not zero. The specific transformation depends on the offset between the current position and the new branch target.

**Safety**:  
It is crucial to ensure that the provided `instruction` parameter is a valid `TBZ` opcode. Incorrect opcodes or assumptions that a different type of instruction is a `TBZ` may lead to undefined behaviour.

**Functionality**:
The `rewrite_tbz` function alters the `TBZ` instruction to accommodate a new target address that is outside of its original range. The target address could be within the same 32KiB range or farther, necessitating different rewriting strategies.

**Example**:

1. **Within 32KiB Range**:
    ```rust
    // Original: tbz x0, #0, #4096
    // Rewritten: tbz x0, #0, #8192
    // Parameters: (old_instruction, old_address, new_address, scratch_reg)
    rewrite_tbz(0x00800036_u32.to_be(), 8192, 4096, Some(17));
    ```

2. **Within 128MiB Range**:
    ```rust
    // Original: tbz x0, #0, #4096
    // Rewritten:
    //   - tbnz x0, #0, #8
    //   - b #0x8000000
    rewrite_tbz(0x00800036_u32.to_be(), 0x8000000, 4096, Some(17));
    ```

3. **Within 4GiB Range Aligned to 4096**:
    ```rust
    // Original: tbz x0, #0, #4096
    // Rewritten:
    //   - tbnz w0, #0, #0xc
    //   - adrp x17, #0x8001000
    //   - br x17
    rewrite_tbz(0x00800036_u32.to_be(), 0x8000000, 0, Some(17));
    ```

4. **Within 4GiB Range with Offset**:
    ```rust
    // Original: tbz x0, #0, #4096
    // Rewritten:
    //    - tbnz w0, #0, #0x10
    //    - adrp x17, #0x8001000
    //    - add x17, x17, #0x512
    //    - br x17
    rewrite_tbz(0x00800036_u32.to_be(), 0x8000512, 0, Some(17));
    ```

5. **Out of 4GiB Range (Move and Branch)**:
    ```rust
    // Original: tbz x0, #0, #4096
    // Rewritten:
    //    - tbnz w0, #0, #0x14
    //    - movz x17, #0x1000
    //    - movk x17, #0, lsl #16
    //    - movk x17, #0x1, lsl #32
    //    - br x17
    rewrite_tbz(0x00800036_u32.to_be(), 0x100000000, 0, Some(17));
    ```
