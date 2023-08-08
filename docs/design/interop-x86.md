# Interoperability (x86)

!!! note "This page just contains some small notes and tidbits regarding interoperability."

!!! note "On x64 platforms [e.g. macOS & bare metal] which cannot encode an absolute jump `FF 25` [no mem in 2GB], mov into scratch register and call by reg (~12 bytes)."

To ensure maximum compatibility with existing hooking systems, Reloaded.Hooks uses `5 byte relative jumps` 
(`E9`).  

This is the most common approach in the x86 land, has low overhead and is the most understood approach.  

In the very, very, unlikely event that this is not possible (target is further than 2GB away), 
the following strategy is used:  

- If no existing hook exists, a `6 byte absolute jump` (`FF 25`) will be used (if possible).  
- Otherwise if there is an existing hook:  
    - If we have any allocated buffer in range, insert `5 byte` `E9` jump, and inside wrapper/stub 
      use absolute jump (`FF 25`) if needed.  
    - Otherwise (if possible), use [available free space from function alignment](#fallback-strategy-free-space-from-function-alignment) to store a `FF 25` absolute jump.  
        - If on x64, use `RIP relative jmp` if possible.  
    - Otherwise patch directly with absolute jump `FF 25`, and attempt 
      [return address patching](#fallback-strategy-return-address-patching).  

## Requirements for External Libraries to Interoperate

!!! note "While I haven't studied the source code of other hooking libraries before, I've had no issues in the past with the common [Detours][detours] and [minhook][minhook] libraries that are commonly used"

### Hooking Over Reloaded Hooks 

!!! info "Libraries which can safely interoperate (stack hooks ontop) of Reloaded Hooks Hooks' must satisfy the following."

- Must be able to patch (re-adjust) `E9` relative jump.  
    - In some cases when assembling call to original function, relative jump target may be out of range,
      compatible hooking software must handle this edge case.

- Must be able to automatically determine number of bytes to steal from original function.  
    - This makes it possible to interoperate with the rare times we do a `FF 25` absolute jump when 
      it may not be possible to do a relative jump (i.e.) as we cannot allocate memory in close
      enough proximity. 

### Reloaded Hooks hooking over Existing Hooks

In the past, `Reloaded.Hooks` handled patching the stolen assembly instructions from the prologue
of the original function manually. 

These days, it's handled automatically by [iced] library.
This will handle all cases.

#### Fallback Strategy: Free Space from Function Alignment

!!! info "x86 processors typically fetch instructions 16 byte boundaries."

!!! info "To optimise for this, compilers pad the space between end of last function and start of next (usually with `0xCC` or `0x90`)"

!!! tip "We can exploit this for code space 😉"

If there's sufficient padding before the function, we can insert our absolute jump there if needed,
and do a short relative jump to it.

#### Fallback Strategy: Return Address Patching

!!! note "This section explains how Reloaded handles an edge case within an already super rare case."

For any of this to take place, the following conditions must be true:  
- An existing `E9` hook exists.  
- Reloaded can't find free memory within `2GB` range of this hook.  
  - The existing hook was somehow able to find free memory in this range, but we can't...  
- [Free Space from Function Alignment Strategy](#fallback-strategy-free-space-from-function-alignment) fails.  
- The instructions at beginning of the hooked function happened to just perfectly align such that 5 bytes.  

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

## Additional Design Drivers

### Some hooking libraries don't cleanup remaining stolen bytes after installing.

!!! note "Notably Steam does this for rendering (overlay) and input (controller support)."

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

This is *extremely dangerous* for the super rare case we need to do [return address patching](#return-address-patching) 
[i.e. when there is an existing `0xE9` hook and no memory within +-2GB is available].  

[detours]: https://github.com/microsoft/Detours
[iced]: https://github.com/icedland/iced
[minhook]: https://github.com/TsudaKageyu/minhook.git