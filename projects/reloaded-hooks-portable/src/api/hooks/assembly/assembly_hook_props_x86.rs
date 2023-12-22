use bitfield::bitfield;

bitfield! {
    /// Defines the data layout of the Assembly Hook data for x86.
    pub struct AssemblyHookPackedProps(u32);
    impl Debug;

    /// True if the hook is enabled, else false.
    pub is_enabled, set_is_enabled: 0;

    /// If true, 'branch to hook' array is 5 bytes instead of 2 bytes on x86.
    is_long_branch_to_hook_len, set_is_long_branch_to_hook_len: 1;

    /// If true, 'branch to orig' array is 5 bytes instead of 2 bytes on x86.
    is_long_branch_to_orig_len, set_is_long_branch_to_orig_len: 2;

    /// Length of the 'disabled code' array.
    u16, disabled_code_len, set_disabled_code_len_impl: 14, 3; // Max 4KiB.

    /// Length of the 'enabled code' array.
    u32, enabled_code_len, set_enabled_code_len_impl: 31, 15; // Max 128KiB.
}

impl AssemblyHookPackedProps {
    /// Gets the length of the 'enabled code' array.
    pub fn get_enabled_code_len(&self) -> usize {
        self.enabled_code_len() as usize
    }

    /// Gets the length of the 'disabled code' array.
    pub fn get_disabled_code_len(&self) -> usize {
        self.disabled_code_len() as usize
    }

    /// Sets the length of the 'enabled code' array with a minimum value of 1.
    pub fn set_enabled_code_len(&mut self, len: usize) {
        debug_assert!(len >= 1, "Length must be at least 1");
        self.set_enabled_code_len_impl(len as u32);
    }

    /// Sets the length of the 'disabled code' array with a minimum value of 1.
    pub fn set_disabled_code_len(&mut self, len: usize) {
        debug_assert!(len >= 1, "Length must be at least 1");
        self.set_disabled_code_len_impl(len as u16);
    }

    /// Sets the 'branch to orig' length field based on the provided length.
    pub fn set_branch_to_hook_len(&mut self, len: usize) {
        debug_assert!(len == 2 || len == 5, "Length must be either 2 or 5");
        self.set_is_long_branch_to_hook_len(len == 5);
    }

    /// Gets the length of the 'branch to hook' array.
    pub fn get_branch_to_hook_len(&self) -> usize {
        if self.is_long_branch_to_hook_len() {
            5
        } else {
            2
        }
    }

    /// Sets the 'branch to orig' length field based on the provided length.
    pub fn set_branch_to_orig_len(&mut self, len: usize) {
        debug_assert!(len == 2 || len == 5, "Length must be either 2 or 5");
        self.set_is_long_branch_to_orig_len(len == 5);
    }

    /// Gets the length of the 'branch to orig' array.
    pub fn get_branch_to_orig_len(&self) -> usize {
        if self.is_long_branch_to_orig_len() {
            5
        } else {
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enabled_and_disabled_code_lengths() {
        let mut props = AssemblyHookPackedProps(0);

        // Test setting and getting enabled code length
        props.set_enabled_code_len(123);
        assert_eq!(props.get_enabled_code_len(), 123);

        // Test setting and getting disabled code length
        props.set_disabled_code_len(456);
        assert_eq!(props.get_disabled_code_len(), 456);

        // Test minimum length enforcement
        props.set_enabled_code_len(1);
        props.set_disabled_code_len(1);
        assert_eq!(props.enabled_code_len(), 1);
        assert_eq!(props.disabled_code_len(), 1);
    }

    #[test]
    #[should_panic(expected = "Length must be at least 1")]
    fn test_enabled_code_length_minimum() {
        let mut props = AssemblyHookPackedProps(0);
        props.set_enabled_code_len(0); // Should panic
    }

    #[test]
    #[should_panic(expected = "Length must be at least 1")]
    fn test_disabled_code_length_minimum() {
        let mut props = AssemblyHookPackedProps(0);
        props.set_disabled_code_len(0); // Should panic
    }

    #[test]
    fn test_branch_lengths() {
        let mut props = AssemblyHookPackedProps(0);

        // Test setting and getting branch to hook length
        props.set_branch_to_hook_len(2);
        assert_eq!(props.get_branch_to_hook_len(), 2);
        props.set_branch_to_hook_len(5);
        assert_eq!(props.get_branch_to_hook_len(), 5);

        // Test setting and getting branch to orig length
        props.set_branch_to_orig_len(2);
        assert_eq!(props.get_branch_to_orig_len(), 2);
        props.set_branch_to_orig_len(5);
        assert_eq!(props.get_branch_to_orig_len(), 5);
    }

    #[test]
    #[should_panic(expected = "Length must be either 2 or 5")]
    fn test_invalid_branch_to_hook_length() {
        let mut props = AssemblyHookPackedProps(0);
        props.set_branch_to_hook_len(3); // Should panic
    }

    #[test]
    #[should_panic(expected = "Length must be either 2 or 5")]
    fn test_invalid_branch_to_orig_length() {
        let mut props = AssemblyHookPackedProps(0);
        props.set_branch_to_orig_len(4); // Should panic
    }
}
