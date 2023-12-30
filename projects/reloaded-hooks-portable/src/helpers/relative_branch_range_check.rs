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
