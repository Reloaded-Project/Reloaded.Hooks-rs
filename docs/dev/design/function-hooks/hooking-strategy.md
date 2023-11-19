# Interoperability (General)

!!! note "This page just contains common information regarding interoperability that are common to all platforms."

Interpoerability in this sense means 'stacking hooks ontop of other libraries', and how other libraries
can stack hooks ontop of `reloaded-hooks-rs`.

## General Hooking Strategy

!!! info "This is the general hooking strategy employed by `reloaded-hooks`; derived from the facts in the rest of this document."

To ensure maximum compatibility with existing hooking systems, `reloaded-hooks` uses 
[relative jumps](../../arch/operations.md#jumprelative) as these are the most popular,
and thus best supported by other libraries when it comes to hook stacking.  

These are the lowest overhead jumps, so are preferable in any case. 

### If Relative Jump is Not Possible

In the very, very, unlikely event that using (target is further than 
`max relative jump distance`), the following strategy below is used.

#### No Existing Hook

If no existing hook exists, an [absolute jump](../../arch/operations.md#jumpabsolute) will be used (if possible).  
- Prefer [indirect absolute jump](../../arch/operations.md#jumpabsoluteindirect) (if possible).  

!!! note "We check for presence of 'existing hook' by catching some common instruction patterns."

#### Existing Hook

- If we have any allocated buffer in range, insert [relative jump](../../arch/operations.md#jumprelative), 
  and inside wrapper/stub use [absolute jump](../../arch/operations.md#jumpabsolute) if needed.  
    - This prevents [your hook longer than original error case](#when-your-hook-is-longer-than-original).  

- Otherwise (if possible), use [available free space from function alignment](#fallback-strategy-free-space-from-function-alignment).  
    - If supported [IP Relative Jmp](../../arch/operations.md#jumpiprelative), with target address in free space.  
    - Otherwise try store whole [absolute jump](../../arch/operations.md#jumpabsolute), in said alignment space.

- Otherwise use [absolute jump](../../arch/operations.md#jumpabsolute).
    - And attempt [return address patching](#return-address-patching), if this is ever re-implemented into library.  

### Calling Back into Original Function

In order to optimize the [code relocation](../../arch/overview.md#code-relocation) process, `reloaded-hooks`, 
will try to find a buffer that's within relative jump range to the original jump target.

If this is not possible, `reloaded-hooks` will start rewriting [relative jump(s)](../../arch/operations.md#jumprelative) 
from the original function to [absolute jump(s)](../../arch/operations.md#jumpabsolute) in the presence
of recognised patterns; if the code rewriter supports this.

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

And then, a large hook using an [absolute jump](../../arch/operations.md#jumpabsolute) with register is applied:

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

The invalid instructions will now become part of the 'stolen' bytes, [when you call the original](./overview.md#when-activated); 
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

!!! info "Strategies used for improving interoperability with other hooks."

### Free Space from Function Alignment

!!! info "This is a strategy for encoding [absolute jumps](../../arch//operations.md#callabsolute) using fewer instructions."

!!! info "Processors typically fetch instructions 16 byte boundaries."

!!! info "To optimise for this, compilers pad the space between end of last function and start of next."

!!! tip "We can exploit this 😉"

If there's sufficient padding before the function, we can:
- Insert our absolute jump there, and branch to it.  
or
- Insert jump target there, and branch using that jump target.  

### Return Address Patching

!!! warning "This feature will not be ported over from legacy `Reloaded.Hooks`, until an edge case is found that requires this."

!!! note "This section explains how Reloaded handles an edge case within an already super rare case."

!!! note "This topic is a bit more complex, so we will use x86 as example here."

For any of this to be necessary, the following conditions must be true:  

- An existing [relative jump](../../arch/operations.md#jumprelative) hook exists.  
- Reloaded can't find free memory within [relative jump](../../arch/operations.md#jumprelative) range.  
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

- Must be able to patch (re-adjust) [relative jumps](../../arch/operations.md#jumprelative).  
    - In some cases when assembling call to original function, relative jump target may be out of range,
      compatible hooking software must handle this edge case.

- Must be able to automatically determine number of bytes to steal from original function.  
    - This makes it possible to interoperate with the rare times we do a [absolute jump](../../arch/operations.md#jumpabsolute) when 
      it may not be possible to do a relative jump (i.e.) as we cannot allocate memory in close
      enough proximity. 

### Reloaded Hooks hooking over Existing Hooks

!!! info "See: [Code Relocation](../../arch/overview.md#code-relocation)"

[detours]: https://github.com/microsoft/Detours
[minhook]: https://github.com/TsudaKageyu/minhook.git