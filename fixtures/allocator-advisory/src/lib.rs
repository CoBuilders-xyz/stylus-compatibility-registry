extern crate stylus_sdk;
extern crate wee_alloc;

// Depending on an allocator implementation does not register it globally.
pub fn allocator_size() -> usize {
    core::mem::size_of::<wee_alloc::WeeAlloc>()
}
