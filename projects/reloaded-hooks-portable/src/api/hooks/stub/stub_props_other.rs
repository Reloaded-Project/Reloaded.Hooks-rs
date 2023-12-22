use bitfield::bitfield;

bitfield! {
    /// Defines the data layout of the Assembly Hook data for architectures
    /// with variable length instructions.
    pub struct StubPackedProps(u32);
    impl Debug;

    /// True if the hook is enabled, else false.
    pub is_enabled, set_is_enabled: 0;

    reserved, _: 1;

    /// Size of the 'swap' space where hook function and original code are swapped out.
    u16, swap_size, set_swap_size_impl: 16, 2; // Max 32KiB.

    /// Size of the 'swap' space where hook function and original code are swapped out.
    u16, hook_fn_size, set_hook_fn_size_impl: 31, 17; // Max 32KiB.
}

impl StubPackedProps {
    pub fn get_swap_size(&self) -> usize {
        self.swap_size() as usize
    }

    pub fn set_swap_size(&mut self, size: usize) {
        debug_assert!(size <= 32 * 1024, "Swap size must be at most 32KiB");
        self.set_swap_size_impl(size as u16);
    }

    pub fn get_hook_fn_size(&self) -> usize {
        self.hook_fn_size() as usize
    }

    pub fn set_hook_fn_size(&mut self, size: usize) {
        debug_assert!(
            size <= 32 * 1024,
            "Hook function size must be at most 32KiB"
        );
        self.set_hook_fn_size_impl(size as u16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_and_hook_fn_sizes() {
        let mut props = StubPackedProps(0);

        // Test setting and getting swap size
        props.set_swap_size(1024);
        assert_eq!(props.get_swap_size(), 1024);

        // Test setting and getting hook function size
        props.set_hook_fn_size(2048);
        assert_eq!(props.get_hook_fn_size(), 2048);

        // Test upper limits
        props.set_swap_size(32 * 1024 - 1);
        props.set_hook_fn_size(32 * 1024 - 1);
        assert_eq!(props.get_swap_size(), 32 * 1024 - 1);
        assert_eq!(props.get_hook_fn_size(), 32 * 1024 - 1);
    }

    #[test]
    #[should_panic(expected = "Swap size must be at most 32KiB")]
    fn test_swap_size_limit() {
        let mut props = StubPackedProps(0);
        props.set_swap_size(33 * 1024); // Should panic
    }

    #[test]
    #[should_panic(expected = "Hook function size must be at most 32KiB")]
    fn test_hook_fn_size_limit() {
        let mut props = StubPackedProps(0);
        props.set_hook_fn_size(33 * 1024); // Should panic
    }
}
