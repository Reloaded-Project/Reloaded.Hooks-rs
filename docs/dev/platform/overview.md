# Platform Overview

!!! info "This page provides a list of platform specific functionality required for supporting `Reloaded.Hooks-rs`."

- `Required` means library must have this to function.  
- `Recommended` means library may not work on some edge cases.  
- `Optional` means library can function without it.  

!!! tip "To add support for new platforms, supply the necessary function pointers in `platform_functions.rs`."

| Feature                                                               | Windows | Linux | macOS |
| --------------------------------------------------------------------- | ------- | ----- | ----- |
| [Permission Change](#required-permission-change)                      | ✅       | ✅     | ✅     |
| [W^X Disable/Restore](#required-wx-disablerestore)                    | N/A       | ✅ [1]     | ❌ [2]     |
| [Targeted Memory Allocation](#recommended-targeted-memory-allocation) | ✅       | ✅     | ✅     |

[1] May be present depending on kernel configuration. Have not done adequate research.  
[2] Needed for Apple Silicon only? [Open Issue](https://github.com/Reloaded-Project/Reloaded.Hooks-rs/issues/1)

## How to Implement

!!! tip "The library provides a `platform_functions.rs` file which contains all the platform specific functions."

It's recommended you submit a PR to add support for your platform. If your platform is very 
custom/esoteric, you may alternatively replace the pointers in `platform_functions.rs` with your 
own implementation, that will work too.

## (Required) Permission Change

!!! info "Many platforms have per-page access permissions; which may prevent certain regions of memory from being modified."

Notably for the use cases of this library, the `.text` section is usually non-writeable, which 
prevents hooking app functions out of the box.  

To work around this, the library will call the `unprotect` function in `platform_functions.rs` before applying
a function and `protect` function to restore protection. For non-common OSes, you must replace these functions
with your own implementation(s).

For the common operating systems; the `protect`/`unprotect` functions map to the following API calls:  

- Windows: `VirtualProtect`  
- Linux & macOS: `mprotect`  

## (Required) W^X Disable/Restore

!!! note "Only affects some platforms."

!!! info 

    Some platforms enforce a security protection called 'Write XOR Execute'; where a memory page may only be marked as writeable
    OR executable at any moment in time.

- [Relevant Issue for macOS M1](https://github.com/Reloaded-Project/Reloaded.Hooks-rs/issues/1)

To work around this, the library will call the `disable_write_xor_execute` function in `platform_functions.rs` before making changes and call `restore_write_xor_execute` after.

## (Recommended) Targeted Memory Allocation

!!! info

    The process of [code relocation](../arch/overview.md#code-relocation) might require that new location of the code
    is within a certain region of the old code, usually 2GB or 4GB (depending on platform).

In this case, you must walk over the memory pages of a process; and find a suitable place to allocate 😉
