# Code Relocation

!!! info "This page provides a listing of all instructions rewritten as part of the [Code Relocation](../overview.md#code-relocation) process."

## ADR(P)

**Purpose**:  

The `ADR` instruction in ARM architectures computes the address of a label and writes it to the destination register. This utility modifies the `ADR` instruction's encoding to adjust for a new memory location, facilitating relocation or code injection.

**Behavior**:  

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
The `Bcc` instruction in ARM architectures performs a conditional branch based on specific condition flags. This function modifies the `Bcc` instruction's encoding to adjust for a new memory location, making it apt for relocation or code injection.

**Behavior**:  
The Branch Conditional instruction is rewritten as:
- BCC
- BCC <skip> + [B]
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

1. **Last Resort**:
    ```rust
    // Before: b.eq #0
    // After: 
    //   - movz x17, #0 
    //   - b.ne #0xc
    //   - br x17
    rewrite_bcc(0x00000054_u32.to_be(), 0, 0x8000000, Some(17));
    ```