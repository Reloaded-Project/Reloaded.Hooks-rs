# Common Design Notes

!!! info "Design notes common to all hooking strategies."

## Wrappers

### Wrapper

!!! info "Wrappers are stubs which convert from the calling convention of the original function to your calling convention."

!!! note "If the calling convention of the hooked function and your function matches, this wrapper is simply just 1 `jmp` instruction."

Wrappers are documented [in their own page here](./wrappers.md).

### ReverseWrapper

!!! info "Stub which converts from your code's calling convention to original function's calling convention"

!!! info "This is basically [Wrapper](#wrappers) with `source` and `destination` swapped around"

## Hook Memory Layouts & Thread Safety

!!! info "Hooks in `reloaded-hooks-rs` are structured in a very specific way to ensure thread safety."

!!! info "They sacrifice a bit of memory usage in favour of performance + thread safety."

Most hooks, regardless of type have a memory layout that looks something like this:

```rust
// Size: 2 registers
pub struct Hook
{
    /// The address of the stub containing bridging code
    /// between your code and custom code. This is the address
    /// of the code that will actually be executed at runtime.
    stub_address: usize,

    /// Address of the 'properties' structure, containing
    /// the necessary info to manipulate the data at stub_address
    props: NonNull<StubPackedProps>,
}
```

Notably, there are two heap allocations. One at `stub_address`, which contains the executable code,
and one at `props`, which contains packed info of the stub at `stub_address`.

The hooks use a 'swapping' system. Both `stub_address` and `props` contains `swap space`. When you
enable or disable a hook, the data in the two 'swap spaces' are swapped around. 

In other words, when `stub_address`' 'swap space' contains the code for `HookFunction` (hook enabled), 
the 'swap space' at `props`' contains the code for `Original Code`.

Thread safety is ensured by making writes within the stub itself atomic, as well as making the emplacing
of the jump to the stub in the original application code atomic.

### Stub Layout

!!! info "The memory region containing the actual executed code."

The stub has two possible layouts, if the `Swap Space` is small enough such that it can be atomically
overwritten, it will look like this:

```text
- 'Swap Space' [HookCode / OriginalCode]
<pad to atomic register size>
```

Otherwise, if `Swap Space` cannot be atomically overwritten, it will look like:

```text
- 'Swap Space' [HookCode / OriginalCode]
- HookCode
- OriginalCode
```

!!! note "Some hooks may store, extra data after `OriginalCode`."

For example, if calling convention conversion is needed, the `HookCode` becomes a 
[ReverseWrapper](#reversewrapper), and the stub will also contain a [Wrapper](#wrapper).

If calling convention conversion is needed, the layout looks like this:

```
- 'Swap Space' [ReverseWrapper / OriginalCode]
- ReverseWrapper
- OriginalCode
- Wrapper
```

#### Example (When Atomically Overwriteable)

!!! info "Using ARM64 [Assembly Hook](./assembly-hooks/overview.md) as an example."

If the *'OriginalCode'* was:

```asm
mov x0, x1
add x0, x2
```

And the *'HookCode'* was:

```asm
add x1, x1
mov x0, x2
```

Since the size of the swap space is less than 16 bytes (assuming 4 byte instructions),
the memory would look like this when the hook is enabled:

```asm
swap: ; Currently Applied (Hook)
    add x1, x1
    mov x0, x2
    b back_to_code ; 12 bytes total
```

#### Example (Not Atomically Overwriteable)

Now, let's consider an example where the swap space is larger than the amount
of bytes that can be atomically written (over 16 bytes, in this ARM64 case)

If the *'OriginalCode'* was:

```asm
mov x0, x1
add x0, x2
sub x0, x3
mul x0, x4
add x0, x5
```

And the *'HookCode'* was:

```asm
add x1, x1
mov x0, x2
sub x1, x3
mul x1, x4
add x1, x5
```

The memory would look like this when the hook is enabled:

```asm
swap: ; Currently Applied (Hook)
    add x1, x1
    mov x0, x2
    sub x1, x3
    mul x1, x4
    add x1, x5
    b back_to_code ; 24 bytes total

hook: ; HookCode
    add x1, x1
    mov x0, x2
    sub x1, x3
    mul x1, x4
    add x1, x5
    b back_to_code

original: ; OriginalCode
    mov x0, x1
    add x0, x2
    sub x0, x3
    mul x0, x4
    add x0, x5
    b back_to_code
```

Therefore, the `hook` and `original` code are stored separately. When the hook is being enabled/disabled the
`swap` space will contain a temporary branch to either the `hook` or `original` before being overwritten.
(To support atomic hook/unhook)

### Heap (Props) Layout

Each Assembly Hook contains a pointer to the heap stub (seen above) and a pointer to the heap.

The heap contains all information required to perform operations on the stub.

```text
- StubPackedProps
    - Enabled Flag
    - IsSwapOnly
    - SwapSize
    - HookSize
- [Hook Function / Original Code]
```

The data in the heap contains a short `StubPackedProps`` struct, detailing the data stored over in the
stub. 

The `SwapSize` contains the length of the 'swap' info (and also consequently, offset of `HookCode`).  
The `HookSize` contains the length of the 'hook' instructions (and consequently, offset of `OriginalCode`).  

If the `IsSwapOnly` flag is set, then this data is to be atomically overwritten.

### The 'Enable' / 'Disable' Process

!!! info "When transitioning between Enabled/Disabled state, we place a temporary branch at `entry`, this allows us to manipulate the remaining code safely."

!!! info "Using ARM64 [Assembly Hook](./assembly-hooks/overview.md) as an example."

We start the 'disable' process with a temporary branch:

```asm
entry: ; Currently Applied (Hook)
    b original ; Temp branch to original
    mov x0, x2
    b back_to_code

hook: ; Backup (Hook)
    add x1, x1
    mov x0, x2
    b back_to_code

original: ; Backup (Original)
    mov x0, x1
    add x0, x2
    b back_to_code
```

!!! note "Don't forget to clear instruction cache on non-x86 architectures which need it."

This ensures we can safely overwrite the remaining code...

Then we overwrite `entry` code with `hook` code, except the branch:

```asm
entry: ; Currently Applied (Hook)
    b original     ; Branch to original
    add x0, x2     ; overwritten with 'original' code.
    b back_to_code ; overwritten with 'original' code.

hook: ; Backup (Hook)
    add x1, x1
    mov x0, x2
    b back_to_code

original: ; Backup (Original)
    mov x0, x1
    add x0, x2
    b back_to_code
```

And lastly, overwrite the branch. 

To do this, read the original `sizeof(nint)` bytes at `entry`, replace branch bytes with original bytes 
and do an atomic write. This way, the remaining instruction is safely replaced.

```asm
entry: ; Currently Applied (Hook)
    add x1, x1     ; 'original' code.
    add x0, x2     ; 'original' code.
    b back_to_code ; 'original' code.

original: ; Backup (Original)
    mov x0, x1
    add x0, x2
    b back_to_code

hook: ; Backup (Hook)
    add x1, x1
    mov x0, x2
    b back_to_code
```

This way we achieve zero overhead CPU-wise, at expense of some memory.

### Limits

Stub info is packed by default to save on memory space. By default, the following limits apply:

| Property             | 4 Byte Instruction (e.g. ARM64) | Other (e.g. x86) |
| -------------------- | ------------------------------- | ---------------- |
| Max Orig Code Length | 128KiB                          | 32KiB            |
| Max Hook Code Length | 128KiB                          | 32KiB            |

!!! note "These limits may increase in the future if additional required functionality warrants extending metadata length."

## Thread Safety on x86

!!! note "Thread safety is ***'theoretically'*** not guaranteed for every possible x86 processor, however is satisfied for all modern CPUs."

!!! tip "The information below is x86 specific but applies to all architectures with a non-fixed instruction size. Architectures with fixed instruction sizes (e.g. ARM) are thread safe in this library by default."

### The Theory

> If the `jmp` instruction emplaced when [switching state](./assembly-hooks/overview.md#switching-state) overwrites what originally
  were multiple instructions, it is *theoretically* possible that the placing the `jmp` will make the
  instruction about to be executed invalid.

For example if the previous instruction sequence was:

```asm
0x0: push ebp
0x1: mov ebp, esp ; 2 bytes
```

And inserting a jmp produces:

```asm
0x0: jmp disabled ; 2 bytes
```

It's possible that the CPU's Instruction Pointer was at `0x1` at the time of the overwrite, making the
`mov ebp, esp` instruction invalid.

### What Happens in Practice

!!! tip "In practice, modern x86 CPUs (1990 onwards) from Intel, AMD and VIA prefetch instruction in batches of 16 bytes."

    And in the recent years, this has been increased to 32 bytes.

We place our stubs generated by the various hooks on 32-byte boundaries for this 
(and optimisation) reasons.

So, by the time we change the code, the CPU has already prefetched the instructions we are atomically 
overwriting.

In other words, it is simply not possible to perfectly time a write such that a thread at
Instruction Pointer `0x1` (`mov ebp, esp`) [as in example above] would read an invalid instruction.

Because that instruction was prefetched and is being executed from local thread cache.

### What is Safe

Here is a thread safety table for x86, taking the above into account:

| Safe? | Hook     | Notes                                                                                          |
| ----- | -------- | ---------------------------------------------------------------------------------------------- |
| ✅     | Function | Functions start on multiples of 16 on pretty much all compilers, per Intel Optimisation Guide. |
| ✅     | Branch   | Stubs are 16 aligned.                                                                          |
| ✅     | Assembly | Stubs are 16 aligned.                                                                          |
| ✅     | VTable   | VTable entries are `usize` aligned, and don't cross cache boundaries.                          |

## Hook Length Mismatch Problem

!!! info "When a hook is already present, and you wish to stack that hook over the existing hook, certain problems might arise."

### When your hook is shorter than original.

!!! tip "This is notably an issue when a hook entry composes of more than 1 instruction; i.e. on RISC architectures."

!!! info "There is a potential register allocation caveat in this scenario."

Pretend you have the following ARM64 function:

=== "ARM64"

    ```asm
    ADD x1, #5
    ADD x2, #10
    ADD x0, x1, x2
    ADD x0, x0, x0
    RET
    ```

=== "C"

    ```asm
    x1 = x1 + 5;
    x2 = x2 + 10;
    int x0 = x1 + x2;
    x0 = x0 + x0;
    return x0;
    ```

And then, a large hook using an [absolute jump](../arch/operations.md#jumpabsolute) with register is applied:

```asm
# Original instructions here replaced
MOVZ x0, A
MOVK x0, B, LSL #16
MOVK x0, C, LSL #32
MOVK x0, D, LSL #48
B x0
# <= branch returns here
```

If you then try to apply a smaller hook after applying the large hook, you might run into the following situation:

```asm
# The 3 instructions here are an absolute jump using pointer.
adrp x9, [0]        
ldr x9, [x9, 0x200] 
br x9
# Call to original function returns here, back to then branch to previous hook
MOVK x0, D, LSL #48
B x0
```

This is problematic, with respect to register allocation. 
Absolute jumps on some RISC platforms like ARM will always require the use of a scratch register. 

But there is a risk the scratch register used is the same register (`x0`) as the register used by the
previous hook as the scratch register. In which case, the jump target becomes invalid.

#### Resolution Strategy

- Prefer absolute jumps without scratch registers (if possible).  
- Detect `mov` + `branch` combinations for each target architecture.
    - And *extend* the function's stolen bytes to cover the entirety.
    - This avoids the scratch register duplication issue, as original hook code will branch to its own
    code before we end up using the same scratch register.

### When your hook is longer than original.

!!! note "Only applies to architectures with variable length instructions. (x86)"

!!! info "Some hooking libraries don't clean up remaining stolen bytes after installing a hook."

!!! note "Very notably Steam does this for rendering (overlay) and input (controller support)."

Consider the original function having the following instructions:

```
48 8B C4      mov rax, rsp
48 89 58 08   mov [rax + 08], rbx
```

After Steam hooks, it will leave the function like this

```
E9 XX XX XX XX    jmp 'somewhere'
58 08             <invalid instruction. leftover from state before>
```

If you're not able to install a relative hook, e.g. need to use an absolute jump

```
FF 25 XX XX XX XX    jmp ['addr']
```

The invalid instructions will now become part of the 'stolen' bytes, [when you call the original](./function-hooks/overview.md#when-activated); 
and invalid instructions may be executed.

#### Resolution Strategy

This library must do the following:  

- Prefer shorter hooks (`relative jump` over `absolute jump`) when possible.  
- Leave nop(s) after placing any branches, to avoid leaving invalid instructions.
    - Don't contribute to the problem.   

There unfortunately isn't much we can do to detect invalid instructions generated by other hooking libraries
reliably, best we can do is try to avoid it by using shorter hooks. Thankfully this is not a common issue
given most people use the 'popular' libraries.

## Fallback Strategies

### Return Address Patching

!!! warning "This feature will not be ported over from legacy `Reloaded.Hooks`, until an edge case is found that requires this."

!!! note "This section explains how Reloaded handles an edge case within an already super rare case."

!!! note "This topic is a bit more complex, so we will use x86 as example here."

For any of this to be necessary, the following conditions must be true:  

- An existing [relative jump](../arch/operations.md#jumprelative) hook exists.  
- Reloaded can't find free memory within [relative jump](../arch/operations.md#jumprelative) range.  
  - The existing hook was somehow able to find free memory in this range, but we can't...  (<= main reason this is improbable!!)
- [Free Space from Function Alignment Strategy](#free-space-from-function-alignment) fails.  
- The instructions at beginning of the hooked function happened to just perfectly align such that our hook
  jump is longer than the existing one.  

The low probability of this happening, at least on Windows and/or Linux is rather insane. It cannot
be estimated, but if I were to have a guess, maybe 1 in 1 billion. You'd be more likely to die 
from a shark attack.

------------------------------

In any case, when this happens, Reloaded performs *return address patching*.  

Suppose a foreign hooking library hooks a function with the following prologue:

```asm
55        push ebp
89 e5     mov ebp, esp
00 00     add [eax], al
83 ec 20  sub esp, 32 
...
```

After hooking, this code would look like:

```asm
E9 XX XX XX XX  jmp 'somewhere'
<= existing hook jumps back here when calling original (this) function
83 ec 20        sub esp, 32 
...
```

When the prologue is set up 'just right', such that the existing instrucions divide perfectly
into 5 bytes, and we need to insert a 6 byte absolute jmp `FF 25`, Reloaded must patch the return address.

Reloaded has a built in patcher for this super rare scenario, which detects and attempts to patch return
addresses of the following patterns:

```
Where nop* represents 0 or more nops.

1. Relative immediate jumps.       

    nop*
    jmp 0x123456
    nop*

2. Push + Return

    nop*
    push 0x612403
    ret
    nop*

3. RIP Relative Addressing (X64)

    nop*
    JMP [RIP+0]
    nop*
```

This patching mechanism is rather complicated, relies on disassembling code at runtime and thus won't be explained here.

!!! danger "Different hooking libraries use different logic for storing callbacks. In some cases alignment of code (or rather lack thereof) can also make this operation unreliable, since we rely on disassembling the code at runtime to find jumps back to end of hook. ***The success rate of this operation is NOT 100%***"

## Requirements for External Libraries to Interoperate

!!! note "While I haven't studied the source code of other hooking libraries before, I've had no issues in the past with the common [Detours][detours] and [minhook][minhook] libraries that are commonly used"

### Hooking Over Reloaded Hooks 

!!! info "Libraries which can safely interoperate (stack hooks ontop) of Reloaded Hooks Hooks' must satisfy the following."

- Must be able to patch (re-adjust) [relative jumps](../arch/operations.md#jumprelative).  
    - In some cases when assembling call to original function, relative jump target may be out of range,
      compatible hooking software must handle this edge case.

- Must be able to automatically determine number of bytes to steal from original function.  
    - This makes it possible to interoperate with the rare times we do a [absolute jump](../arch/operations.md#jumpabsolute) when 
      it may not be possible to do a relative jump (i.e.) as we cannot allocate memory in close
      enough proximity. 

### Reloaded Hooks hooking over Existing Hooks

!!! info "See: [Code Relocation](../arch/overview.md#code-relocation)"

[detours]: https://github.com/microsoft/Detours
[minhook]: https://github.com/TsudaKageyu/minhook.git