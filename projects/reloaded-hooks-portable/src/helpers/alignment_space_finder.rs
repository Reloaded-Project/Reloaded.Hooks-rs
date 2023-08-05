use core::mem;

/// Finds the amount of available padding space before the prolog of a function.
///
/// In a binary file or a piece of assembled memory, the prolog of a function
/// is often preceded by padding bytes. These padding bytes are typically
/// harmless and exist for alignment purposes, to ensure that the function
/// begins at a particular address for optimization. This function allows to
/// quantify the amount of this padding.
///
/// This function scans memory backwards from the provided starting point,
/// counting the number of consecutive elements matching any of the patterns
/// until it finds a non-matching element. It then calculates the size of the
/// padding space in bytes.
///
/// # Parameters
///
/// - `ptr`: A pointer to the start of the function. The function will scan
/// memory backwards from this point. The function assumes this pointer is
/// valid and that it can safely be dereferenced.
///
/// - `patterns`: A list of elements of type `T` that can be treated as
/// padding. The function compares each memory element to the elements in this
/// list to determine if it is part of the padding. This list should not be
/// empty, as this would mean there is no padding.
///
/// # Returns
///
/// This function returns the size of the padding space in bytes as a `u32`.
/// This size can be zero if no padding space is found.
///
/// # Safety
///
/// This function is `unsafe` as it performs raw pointer dereferencing.
/// It can potentially read invalid memory if the provided pointer or patterns
/// list are incorrect. It also assumes that there is at least 1 byte of data
/// that doesn't match any pattern before the start of the function.
///
/// This should be true in any real executable/binary file.
pub unsafe fn find_alignment_size<T: Sized + Copy + PartialEq>(
    mut ptr: *mut T,
    patterns: &[T],
) -> u32 {
    let start = ptr;
    ptr = ptr.offset(-1);

    while patterns.contains(&*ptr) {
        ptr = ptr.offset(-1);
    }

    (start as usize - ptr as usize - mem::size_of::<T>()) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_no_padding() {
        let mut data: Vec<u32> = vec![0, 0, 1, 2, 3, 0, 0];
        let patterns: Vec<u32> = vec![0];
        let padding_size = unsafe {
            find_alignment_size(
                data.as_mut_ptr().offset(4), // Pointer to the first non-zero element
                &patterns,
            )
        };
        assert_eq!(padding_size, 0);
    }

    #[test]
    fn with_padding_on_boundary() {
        let mut data: Vec<u32> = vec![1, 0, 0, 1, 2, 3, 0, 0];
        let patterns: Vec<u32> = vec![0];
        let padding_size = unsafe {
            find_alignment_size(
                data.as_mut_ptr().offset(3), // Pointer to the first non-zero element
                &patterns,
            )
        };
        assert_eq!(padding_size, 8);
    }

    #[test]
    fn no_padding() {
        let mut data: Vec<u32> = vec![1, 2, 3, 4, 5];
        let patterns: Vec<u32> = vec![0];
        let padding_size = unsafe {
            find_alignment_size(
                data.as_mut_ptr().offset(4), // Pointer to the last element
                &patterns,
            )
        };
        assert_eq!(padding_size, 0);
    }

    #[test]
    fn when_both_patterns_present_1_byte() {
        let mut data: Vec<u8> = vec![1, 0xCC, 0xCC, 0x90, 0x90, 1, 2, 3, 4, 5];
        let patterns: Vec<u8> = vec![0xCC, 0x90];

        let padding_size = unsafe {
            find_alignment_size(
                data.as_mut_ptr().offset(5), // Pointer to the first non-zero element
                &patterns,
            )
        };
        assert_eq!(padding_size, 4);
    }

    #[test]
    fn when_one_pattern_present_1_byte() {
        let mut data: Vec<u8> = vec![1, 0x90, 0x90, 0x90, 0x90, 1, 2, 3, 4, 5];
        let patterns: Vec<u8> = vec![0xCC, 0x90];

        let padding_size = unsafe {
            find_alignment_size(
                data.as_mut_ptr().offset(5), // Pointer to the first non-zero element
                &patterns,
            )
        };
        assert_eq!(padding_size, 4);
    }
}
