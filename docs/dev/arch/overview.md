# Architecture Overview

Lists currently supported architectures and their features.

## Feature Support

!!! info "Lists the currently available library features for different architectures."

| Feature                                                                         | x86 & x64 | ARM64 |
| ------------------------------------------------------------------------------- | --------- | ----- |
| [Basic Function Hooking](#basic-function-hooking)                               | ✅         | ✅     |
| [Code Relocation](#code-relocation)                                             | ✅*        | ✅  |
| [Hook Stacking](#hook-stacking)                                                 | ✅         | ✅     |
| [Calling Convention Wrapper Generation](#calling-convention-wrapper-generation) | ✅         | ✅     |
| [Optimal Wrapper Generation](#optimal-wrapper-generation)                       | ✅         | ✅     |
| [Length Disassembler](#length-disassembler)                                     | ✅         | ✅     |

* x86 should work in all cases, but x64 isn't tested against all 5000+ instructions.

## Required 
### Basic Function Hooking

!!! info "The ability to hook/detour existing application functions."

#### How to Implement

!!! info "Implement a code writer by inheriting the `Jit<TRegister>` trait"

In the writer, implement *at least* the following operations:  

- [JumpRelativeOperation](./operations.md#jumprelativeoperation).  
- [JumpAbsoluteOperation](./operations.md#jumpabsoluteoperation) [needed if platform doesn't support [Targeted Memory Allocation](../platform/overview.md)].  

Your *Platform* must also support [Permission Change](../platform/overview.md#required-permission-change), if it is
applicable to your platform.

### Length Disassembler

!!! info "Length disassembly is the ability to determine instruction lengths at a given address."

A length disassembler determines the minimum amount of instructions (in bytes) needed to copy when hooking
a function.

```rust
/// Disassembles items at `code_address` until the length of instructions
/// is equal to or greater than `min_length`. 
/// 
/// # Returns
/// Returns length of instructions (in bytes) greater than or equal to min_length
fn GetHookLength(code_address: usize, min_length: usize) -> usize
```

This is done by disassembling the original instructions at `code_address`, incrementing a length for each
encountered instruction until `length >= min_length`, then returning the result.

#### Example

For [hooking functions](../design/function-hooks/overview.md), it's necessary to inject a `jmp` instruction into the existing code.

For example, given this sequence:

```asm
; x86 Assembly
DoMathWithTwoNumbers:
    cmp rcx, 0 ; 48 83 F9 00
    jg skipAdd ; 7F 0E

    mov rax, [rsp + 8] ; 48 8B 44 24 04
    mov rax, [rsp + 16] ; 48 8B 4C 24 04
    add rax, rcx ; 48 01 C8
    ret ; C3
```

A `5 byte`` relative jump would overwrite the first two instructions, creating:

```asm
; x86 Assembly
DoMathWithTwoNumbers:
    jmp stub ; E9 XX XX XX XX
    <INVALID INSTRUCTION> ; 0E

    mov rax, [rsp + 8] ; 48 8B 44 24 04
    mov rax, [rsp + 16] ; 48 8B 4C 24 04
    add rax, rcx ; 48 01 C8
    ret ; C3
```

When calling the original function again, and thus creating the [Reverse Wrapper](../design/function-hooks/overview.md#key), 
the original instructions overwritten by the `jmp` will need to be executed.

To do this, we must know that the original 2 instructions at `DoMathWithTwoNumbers` were 6, NOT 
`5 byte`s in length total. Such that when we copy the original code to [Reverse Wrapper](../design/function-hooks/overview.md#key)
we get

```asm
cmp rcx, 0 ; 48 83 F9 00
jg skipAdd ; 7F 0E
```

and not

```asm
cmp rcx, 0 ; 48 83 F9 00
<INVALID INSTRUCTION> ; 7F
```

With a length disassembler, we are able to safely copy all the bytes needed.

#### How to Implement

!!! info "Implement a length disassembler by inheriting the `LengthDisassembler` trait."

Use the algorithm described in [example](#example).

### Code Relocation

!!! info "Code relocation is the ability to rewrite existing code such that existing instructions using PC/IP relative operands still have valid operands post patching."

Suppose the following x86 code, which was optimised away to accept first parameter in `ecx` register:  

```c
int DoMathWithTwoNumbers(int operation@ecx, int a, int b) {

    if (operation <= 0) {
        return a + b;
    }

    // Omitted Code Here
}
```

In this case it's possible that there's a jump in the very beginning of the function:  

```asm
DoMathWithTwoNumbers:
    cmp ecx, 0
    jg skipAdd # It's greater than 0

    mov eax, [esp + {wordSize * 1}] ; Left Parameter
    mov ecx, [esp + {wordSize * 2}] ; Right Parameter
    add eax, ecx
    ret

    ; Some Omitted Code Here
    
skipAdd:
    ; Omitted Code Here
```

In a scenario like this, the hooking library would overwrite the `cmp` and `jg` instruction when
it assembles the [hook entry ('enter hook')](../design/function-hooks/overview.md#key); and when the original
function is called again by your hook the, 'wrapper' would now contain this `jg` instruction.

Because `jg` is an instruction relative to the current instruction address, the library must be able
to patch and 'relocate' the function to a new address.

!!! note "Basic code relocation support is needed to [stack hooks](#hook-stacking)."

#### How to Implement

!!! info "Implement a relocator by `CodeRewriter` trait."

There is no 'general strategy' for this, however, here are some pieces of advice:

- Consider looking at the docs for existing relocators (for RISC, [ARM64](../arch/arm64/code_relocation.md)) is a good reference.  
- You will need to rewrite all control flow instructions (`branch` etc.)  
- You will need to rewrite all instructions which are relative to current Instruction Pointer/Program Counter.  
- Use disassembler library (if one exists) for your architecture.  

## Optional (Extras)

### Calling Convention Wrapper Generation

!!! info "The ability to convert between different calling conventions (e.g. `cdecl -> stdcall`)."

To implement this, you implement a code writer by inheriting the `Jit<TRegister>` trait; and 
implement the following operations:  

- [All Non-Optional Instructions](./operations.md).  

### Optimized Wrapper Generation

!!! info "If this is checked, it means the wrappers generate optimal code (to best of knowledge)."

!!! note "While the wrapper generator does most optimisations themselves, in some cases, it may be possible to perform additional optimisations in the JIT/Code Writer side."

For example, the `reloaded-hooks` wrapper generator might generate the following sequence of pushes for ARM64:

```asm
push x0
push x1
```

A clever ARM64 compiler however would be able to translate this to:

```asm
stp x0, x1, [sp, #-16]!
```

For some built in optimisations, like this, you can opt into these specialised instructions with `JitCapabilities` on your `Jit<TRegister>`.

Some others, may be implemented at Jit level instead.

## Misc

### Hook Stacking

!!! info "Hook stacking is the ability to hook a function multiple times."

This should work flawlessly out of the box if all of the [required](#required) elements are implemented.