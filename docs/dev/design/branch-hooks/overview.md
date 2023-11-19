# Branch Hooks

!!! info "Replaces a `branch` (call/jump) to an existing method with a new one."

!!! info "This hook is commonly used when you want to change behaviour of a function, but only for certain callers."

    For example, if you have a method `Draw2DElement` that's used to draw an object to the screen, but
    you only want to move a certain element that's rendered by `Draw2DElement`, you would use a Branch Hook
    to replace `call Draw2DElement` to `call YourOwn2DElement`.

!!! warning "Only guaranteed to work on platforms with [Targeted Memory Allocation](../../platform/overview.md#recommended-targeted-memory-allocation)"

!!! note "I'm not a security person/researcher. I just make full stack game modding tools, mods and libraries. Naming in these design docs might be unconventional."

This hook works by replacing the target of a `call` (a.k.a. Branch with Link) instruction with a new target.

## Comparison with [Function Hook](../function-hooks/overview.md)

!!! tip "A Branch Hook is really a specialised variant of [function hook](../function-hooks/overview.md)."

Notably it differs in the following ways:

- There is no [Wrapper To Call Original Function](../function-hooks/overview.md#when-activated-in-fast-mode) as no instructions are stolen.  
    - Your method will directly call original instead. 

- You `call` the `ReverseWrapper` instead of `jump`ing to it.
- Code replacement is at caller level rather than function level.

## High Level Diagram

### Key

- `Caller Function`: Function which originally called `Original Method`.  
- `ReverseWrapper`: Translates from original function calling convention to yours. Then calls your function.  
- `<Your Function>`: Your Rust/C#/C++/Asm code.
- `Original Method`: Original method to be called.  

### When Activated

```mermaid
flowchart TD
    CF[Caller Function]
    RW[ReverseWrapper]
    HK["&lt;Your Function&gt;"]
    OM[Original Method]

    CF -- "call wrapper" --> RW
    RW -- jump to your code --> HK
    HK -. "Calls &lt;Optionally&gt;" .-> OM
    OM -. "Returns" .-> HK
```

### When Activated in 'Fast Mode'

!!! info "'Fast Mode' is an optimisation that inserts the jmp to point directly into your code when possible."

```mermaid
flowchart TD
    CF[Caller Function]
    HK["&lt;Your Function&gt;"]
    OM[Original Method]

    CF -- "call &lt;<Your Function>&gt; instead of original" --> HK
    HK -. "Calls &lt;Optionally&gt;" .-> OM
    OM -. "Returns" .-> HK
```

This option allows for a small performance improvement, saving 1 instruction and some instruction prefetching load. 

This is on by default (can be disabled), and will take into effect when no conversion between calling conventions is needed and target is within 'Relative Jump' range for your CPU architecture. 

### When Deactivated

```mermaid
flowchart TD
    CF[Caller Function]
    SB[Stub]
    HK[Hook Function]
    OM[Original Method]

    CF -- jump to stub --> SB
    SB -- jump to original --> OM
```

When the hook is deactivated, the stub is replaced with a direct jump back to the original function.

By bypassing your code entirely, it is safe for your dynamic library (`.dll`/`.so`/`.dylib`) 
to unload from the process.