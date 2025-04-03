#![no_std]

#[no_mangle]
pub extern "C" fn ffi_ct_memcmp(lhs: *const u8, rhs: *const u8, len: usize) -> i32 {
    crate::ct_memcmp(lhs, rhs, len)
}
