# Architecture Overview

Lists currently supported architectures.

## Feature Support

!!! info "Lists the currently available library features for different architectures."

| Feature                                                                         | x86 & x64 | ARM64 |
| ------------------------------------------------------------------------------- | --------- | ----- |
| [Basic Function Hooking](#basic-function-hooking)                               | ✅         | ✅     |
| [Code Relocation](#code-relocation)                                             | WIP        | ✅  |
| [Hook Stacking](#hook-stacking)                                                 | ✅         | ✅     |
| [Calling Convention Wrapper Generation](#calling-convention-wrapper-generation) | ✅         | ✅     |
| [Optimal Wrapper Generation](#optimal-wrapper-generation)                       | ✅         | ✅     |

## Basic Function Hooking

!!! info "The ability to hook/detour existing application functions."

### Implementing This

To implement this, you implement a code writer by inheriting the `Jit<TRegister>` trait; and 
implement the following operations:  

- [JumpRelativeOperation](./operations.md#jumprelativeoperation).  
- [JumpAbsoluteOperation](./operations.md#jumpabsoluteoperation) [needed if platform doesn't support [Targeted Memory Allocation](../platform/overview.md)].  

Your *Platform* must also support [Permission Change](../platform/overview.md#required-permission-change), if it is
applicable to your platform.

## Code Relocation

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
it assembles the [hook entry ('enter hook')](../design/overview.md#key); and when the original
function is called again by your hook the, 'wrapper' would now contain this `jg` instruction.

Because `jg` is an instruction relative to the current instruction address, the library must be able
to patch and 'relocate' the function to a new address.

!!! note "Basic code relocation support is needed to [stack hooks](#hook-stacking)."

## Hook Stacking

!!! info "Hook stacking is the ability to hook a function multiple times."

In practice, this is usually supported for all cases.

## Calling Convention Wrapper Generation

To implement this, you implement a code writer by inheriting the `Jit<TRegister>` trait; and 
implement the following operations:  

- [All Non-Optional Instructions](./operations.md).  

## Optimized Wrapper Generation

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

When the code emitter can recognise these patterns, and optimise further, the box is checked.