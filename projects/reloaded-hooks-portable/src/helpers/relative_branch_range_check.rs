use core::ops::Add;

/// Determines if a direct branch is possible between two addresses.
///
/// # Parameters
/// - `hook_address`: The address from which the branch originates.
/// - `new_target`: The target address of the branch.
/// - `max_distance`: Maximum distance that the branch instruction can cover.
/// - `branch_bytes`: Number of bytes the branch instruction occupies.
///
/// # Returns
/// Returns `true` if a direct branch is possible, otherwise `false`.
///
/// # Remarks
/// This implementation is non-architecture specific.
///
/// It doesn't make an assume the architecture either jumps relative to start or end of instruction.
/// Instead, it searches `max_distance` - `branch_bytes` length, to be 'safe'. So depending on the
/// architecture it may be unoptimal by `branch_bytes` bytes.
#[inline]
pub fn can_direct_branch(
    hook_address: usize,
    new_target: usize,
    max_distance: usize,
    branch_bytes: usize,
) -> bool {
    let max_offset = max_distance as isize - branch_bytes as isize;
    let delta = new_target.wrapping_sub(hook_address) as isize;
    delta <= max_offset && delta >= -max_offset
}

/// Determines if a direct branch is possible between two addresses.
///
/// # Parameters
/// - `hook_address`: The address from which the branch originates.
/// - `new_target`: The target address of the branch.
/// - `max_distance`: Maximum distance that the branch instruction can cover.
/// - `branch_bytes`: Number of bytes the branch instruction occupies.
///
/// # Returns
/// Returns `true` if a direct branch is possible, otherwise `false`.
/// Second parameter contains the delta (difference) between end of instruction and `new_target`.
#[inline]
pub fn can_direct_branch_with_distance_from_ins_end(
    hook_address: usize,
    new_target: usize,
    max_distance: usize,
    branch_bytes: usize,
) -> (bool, isize) {
    let max_offset = max_distance as isize;
    let start_pos = hook_address.add(branch_bytes);
    let delta = new_target.wrapping_sub(start_pos) as isize;
    let can_branch = delta <= max_offset && delta >= -max_offset;
    (can_branch, delta)
}
