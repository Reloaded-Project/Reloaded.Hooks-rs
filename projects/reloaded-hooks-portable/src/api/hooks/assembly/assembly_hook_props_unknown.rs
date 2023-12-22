use bitfield::bitfield;

bitfield! {
    /// Defines the data layout of the Assembly Hook data for unknown architectures.
    pub struct AssemblyHookPackedProps(u64);
    impl Debug;

    /// True if the hook is enabled, else false.
    pub is_enabled, set_is_enabled: 0;

    /// Length of 'branch to hook' array.
    u8, branch_to_hook_len, set_branch_to_hook_len_impl: 3, 1;

    /// Length of 'branch to hook' array.
    u8, branch_to_orig_len, set_branch_to_orig_len_impl: 6, 4;

    /// Length of the 'disabled code' array.
    u32, disabled_code_len, set_disabled_code_len_impl: 33, 7; // Max 128MiB.

    /// Length of the 'enabled code' array.
    u32, enabled_code_len, set_enabled_code_len_impl: 63, 34; // Max 1GiB.
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
        self.set_enabled_code_len_impl(len as u32); // Adjust for 'unknown' architecture
    }

    /// Sets the length of the 'disabled code' array with a minimum value of 1.
    pub fn set_disabled_code_len(&mut self, len: usize) {
        debug_assert!(len >= 1, "Length must be at least 1");
        self.set_disabled_code_len_impl(len as u32); // Adjust for 'unknown' architecture
    }

    /// Sets the length of the 'branch to hook' array.
    pub fn set_branch_to_hook_len(&mut self, len: usize) {
        self.set_branch_to_hook_len_impl(len as u8);
    }

    /// Gets the length of the 'branch to hook' array.
    pub fn get_branch_to_hook_len(&self) -> usize {
        self.branch_to_hook_len() as usize
    }

    /// Sets the length of the 'branch to orig' array.
    pub fn set_branch_to_orig_len(&mut self, len: usize) {
        self.set_branch_to_orig_len_impl(len as u8);
    }

    /// Gets the length of the 'branch to orig' array.
    pub fn get_branch_to_orig_len(&self) -> usize {
        self.branch_to_orig_len() as usize
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
        props.set_branch_to_hook_len(3);
        assert_eq!(props.get_branch_to_hook_len(), 3);

        // Test setting and getting branch to orig length
        props.set_branch_to_orig_len(4);
        assert_eq!(props.get_branch_to_orig_len(), 4);
    }
}
