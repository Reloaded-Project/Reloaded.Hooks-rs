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