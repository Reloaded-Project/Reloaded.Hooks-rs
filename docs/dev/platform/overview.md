# Platform Overview

!!! info "This page provides a list of platform specific functionality required for supporting `Reloaded.Hooks-rs`."

- `Required` means library must have this to function.  
- `Recommended` means library may not work on some edge cases.  
- `Optional` means library can function without it.  

!!! tip "To add support for new platforms, supply the necessary function pointers in `platform_functions.rs`."

| Feature                                                               | Windows | Linux | macOS |
| --------------------------------------------------------------------- | ------- | ----- | ----- |
| [Permission Change](#required-permission-change)                      | ✅       | ✅     | ✅     |
| [W^X Disable/Restore](#required-wx-disablerestore)                    | N/A       | N/A [1]     | ⚠️ [2]     |
| [Targeted Memory Allocation](#recommended-targeted-memory-allocation) | ✅       | ✅     | ✅     |

[1] May be present depending on kernel configuration. Have not done adequate research.  
[2] Needed for [Apple Silicon only](https://github.com/Reloaded-Project/Reloaded.Hooks-rs/issues/1).

## How to Implement Support

!!! note "Once you're done, submit a PR to add support for your platform."

### Platform Functions

!!! info "The library provides a `platform_functions.rs` file which contains all the platform specific functions."

Implement the functions in this file for your platform. Generally you'll only need `unprotect_memory`, 
though on some platforms, you may need to implement `disable_write_xor_execute` and `restore_write_xor_execute` 
as well, depending on the platform's security policy.

### (Recommended) Buffers Implementation

!!! tip "For optimal performance, you should add support for your platform to [reloaded-memory-buffers](https://github.com/Reloaded-Project/Reloaded.Memory.Buffers/tree/master/src-rust)."

It's recommended to use `reloaded-hooks-rs` alongside `reloaded-memory-buffers`. The concept of the buffers
library is to perform allocations as close to original code as possible, allowing for more efficient code.

This requires walking memory pages. If your OS does not have a way to do this, you can in the meantime use
the built-in `DefaultBufferFactory`. 

!!! warning "For `DefaultBufferFactory`, you might need to replace `mmap_rs` in `get_any_buffer` to use your platform specific page allocation function."

### Testing Your Implementation

Platform specific functionality is not unit tested as it relies on OS/system state. Instead, integration 
tests are used to test the functionality.

Find the tests for a given hook type (recommend: `assembly_hook` tests) and run them on your platform.  

If you can't run tests on your platform, copy them to one of your programs manually.  

## (Required) Permission Change

!!! info "Many platforms have per-page access permissions; which may prevent certain regions of memory from being modified."

Notably for the use cases of this library, the `.text` section is usually non-writeable, which 
prevents hooking app functions out of the box.  

To work around this, the library will call the `unprotect` function in `platform_functions.rs` before 
making code changes in memory. It will then (for performance reasons) leave the memory unprotected 
for the lifetime of the process (assuming it remains unprotected).

For the common operating systems; the `protect`/`unprotect` functions map to the following API calls:  

- Windows: `VirtualProtect`  
- Linux & macOS: `mprotect`  

## (Required) W^X Disable/Restore

!!! note "Only required on Apple, opt in on Linux/Windows but haven't used in a game software in the wild."

!!! info 

    Some platforms enforce a security protection called 'Write XOR Execute'; where a memory page may only be marked as writeable
    OR executable at any moment in time.

- [Relevant Issue for macOS M1](https://github.com/Reloaded-Project/Reloaded.Hooks-rs/issues/1)

To work around this, the library will call the `disable_write_xor_execute` function in `platform_functions.rs` 
ahead of every function call. It will then call `restore_write_xor_execute` after.

## (Recommended) Targeted Memory Allocation

!!! info

    The process of [code relocation](../arch/overview.md#code-relocation) might require that new location of the code
    is within a certain region of the old code, usually 128MiB, 2GiB or 4GiB (depending on platform).

In this case, you must walk over the memory pages of a process; and find a suitable place to allocate 😉
