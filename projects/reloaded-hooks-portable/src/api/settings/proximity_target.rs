/// Represents a target address within memory for allocation nearness.
///
/// This is used for the allocation of wrappers and other native/interop components.
/// It helps guide memory allocations to be closer to a specific target address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProximityTarget {
    /// Expected item size. Defaults to 256 for safety reasons.
    pub item_size: u32,

    /// Target address near which the allocation is intended.
    pub target_address: usize,

    /// Requested amount of bytes within the target address for the item allocation.
    /// Defaults to the maximum value of `usize` to represent an unlimited range.
    /// This must be greater than `item_size`.
    pub requested_proximity: usize,
}

impl ProximityTarget {
    // Default expected size of assembled item.
    const DEFAULT_ITEM_SIZE: u32 = 128; // default for most platforms

    /// Creates a `ProximityTarget`.
    ///
    /// # Arguments
    /// - `target_address` - The address near which the allocation should be.
    /// - `item_size` - The size of the memory region to allocate.
    /// - `requested_proximity` - The requested proximity to the target address.
    pub fn new(target_address: usize, item_size: u32, requested_proximity: usize) -> Self {
        ProximityTarget {
            target_address,
            item_size,
            requested_proximity,
        }
    }

    /// Creates a `ProximityTarget` with default values, and a requested proximity
    /// to a given address.
    ///
    /// # Arguments
    /// - `target_address` - The address near which the allocation should be.
    /// - `requested_proximity` - The requested proximity to the target address.
    pub fn with_address_and_requested_proximity(
        target_address: usize,
        requested_proximity: usize,
    ) -> Self {
        ProximityTarget {
            target_address,
            item_size: Self::DEFAULT_ITEM_SIZE,
            requested_proximity,
        }
    }

    fn default_target_address() -> usize {
        // There is no cross-platform approach to finding the address of a main module
        // such as the .exe file. On certain platforms, we can do this, but not on others.
        // For now, this is an estimate.
        Self::default_target_address as *const fn() as usize
    }
}
