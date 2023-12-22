use bitfield::bitfield;

bitfield! {
    /// Defines the data layout of the Assembly Hook data for architectures
    /// with fixed instruction sizes of 4 bytes.
    pub struct AssemblyHookPackedProps(u32);
    impl Debug;

    /// True if the hook is enabled, else false.
    pub is_enabled, set_is_enabled: 0;

    reserved, _: 1;

    /// Size of the 'swap' space where hook function and original code are swapped out.
    /// Represented in number of 4-byte instructions.
    u16, swap_size, set_swap_size_impl: 16, 2; // Max 32Ki instructions, 128KiB.

    /// Size of the 'swap' space where hook function and original code are swapped out.
    /// Represented in number of 4-byte instructions.
    u16, hook_fn_size, set_hook_fn_size_impl: 31, 17; // Max 32Ki instructions, 128KiB.
}

impl AssemblyHookPackedProps {
    pub fn get_swap_size(&self) -> usize {
        (self.swap_size() as usize) * 4 // Convert from instructions to bytes
    }

    pub fn set_swap_size(&mut self, size: usize) {
        debug_assert!(
            size % 4 == 0 && size <= 128 * 1024,
            "Swap size must be a multiple of 4 and at most 128KiB"
        );
        self.set_swap_size_impl((size / 4) as u16); // Convert from bytes to instructions
    }

    pub fn get_hook_fn_size(&self) -> usize {
        (self.hook_fn_size() as usize) * 4 // Convert from instructions to bytes
    }

    pub fn set_hook_fn_size(&mut self, size: usize) {
        debug_assert!(
            size % 4 == 0 && size <= 128 * 1024,
            "Hook function size must be a multiple of 4 and at most 128KiB"
        );
        self.set_hook_fn_size_impl((size / 4) as u16); // Convert from bytes to instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_and_hook_fn_sizes() {
        let mut props = AssemblyHookPackedProps(0);

        // Test setting and getting swap size
        props.set_swap_size(1024); // 256 instructions
        assert_eq!(props.get_swap_size(), 1024);

        // Test setting and getting hook function size
        props.set_hook_fn_size(2048); // 512 instructions
        assert_eq!(props.get_hook_fn_size(), 2048);

        // Test upper limits
        props.set_swap_size(127 * 1024); // 32Ki instructions
        props.set_hook_fn_size(127 * 1024); // 32Ki instructions
        assert_eq!(props.get_swap_size(), 127 * 1024);
        assert_eq!(props.get_hook_fn_size(), 127 * 1024);
    }

    #[test]
    #[should_panic(expected = "Swap size must be a multiple of 4 and at most 128KiB")]
    fn test_swap_size_limit() {
        let mut props = AssemblyHookPackedProps(0);
        props.set_swap_size(129 * 1024); // Should panic
    }

    #[test]
    #[should_panic(expected = "Hook function size must be a multiple of 4 and at most 128KiB")]
    fn test_hook_fn_size_limit() {
        let mut props = AssemblyHookPackedProps(0);
        props.set_hook_fn_size(129 * 1024); // Should panic
    }
}
