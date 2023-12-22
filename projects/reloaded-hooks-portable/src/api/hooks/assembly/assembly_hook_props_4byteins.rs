use bitfield::bitfield;

bitfield! {
    /// Defines the data layout of the Assembly Hook data.
    /// For architectures which use 4 byte instructions.
    pub struct AssemblyHookPackedProps(u32);
    impl Debug;

    /// True if the hook is enabled, else false.
    pub is_enabled, set_is_enabled: 0;

    /// Length of the 'disabled code' array.
    u16, disabled_code_len, set_disabled_code_len_impl: 12, 1; // Max 16KiB.

    /// Length of the 'enabled code' array.
    u32, enabled_code_len, set_enabled_code_len_impl: 31, 13; // Max 2MiB.
}

impl AssemblyHookPackedProps {
    /// Gets the length of the 'enabled code' array.
    pub fn get_enabled_code_len(&self) -> usize {
        self.enabled_code_len() as usize * 4
    }

    /// Gets the length of the 'disabled code' array.
    pub fn get_disabled_code_len(&self) -> usize {
        self.disabled_code_len() as usize * 4
    }

    /// Sets the length of the 'enabled code' array with a minimum value of 4.
    pub fn set_enabled_code_len(&mut self, len: usize) {
        debug_assert!(
            len >= 4 && len % 4 == 0,
            "Length must be a multiple of 4 and at least 4"
        );
        self.set_enabled_code_len_impl((len / 4) as u32);
    }

    /// Sets the length of the 'disabled code' array with a minimum value of 4.
    pub fn set_disabled_code_len(&mut self, len: usize) {
        debug_assert!(
            len >= 4 && len % 4 == 0,
            "Length must be a multiple of 4 and at least 4"
        );
        self.set_disabled_code_len_impl((len / 4) as u16);
    }

    /// Sets the 'branch to orig' length field based on the provided length.
    pub fn set_branch_to_hook_len(&mut self, _len: usize) {
        // no-op, for API compatibility
    }

    /// Gets the length of the 'branch to hook' array. Always 4 for AArch64.
    pub fn get_branch_to_hook_len(&self) -> usize {
        4
    }

    /// Sets the 'branch to orig' length field based on the provided length.
    pub fn set_branch_to_orig_len(&mut self, _len: usize) {
        // no-op, for API compatibility
    }

    /// Gets the length of the 'branch to orig' array. Always 4 for AArch64.
    pub fn get_branch_to_orig_len(&self) -> usize {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enabled_and_disabled_code_lengths() {
        let mut props = AssemblyHookPackedProps(0);

        // Test setting and getting enabled code length
        props.set_enabled_code_len(123 * 4); // Multiples of 4
        assert_eq!(props.get_enabled_code_len(), 123 * 4);

        // Test setting and getting disabled code length
        props.set_disabled_code_len(456 * 4); // Multiples of 4
        assert_eq!(props.get_disabled_code_len(), 456 * 4);
    }

    #[test]
    #[should_panic(expected = "Length must be a multiple of 4 and at least 4")]
    fn test_invalid_code_length() {
        let mut props = AssemblyHookPackedProps(0);
        props.set_enabled_code_len(5); // Should panic, not a multiple of 4
    }

    #[test]
    fn test_branch_lengths() {
        let mut props = AssemblyHookPackedProps(0);

        // Setting and getting branch lengths, always 4 for AArch64
        props.set_branch_to_hook_len(4);
        assert_eq!(props.get_branch_to_hook_len(), 4);

        props.set_branch_to_orig_len(4);
        assert_eq!(props.get_branch_to_orig_len(), 4);
    }
}
