#[repr(align(64))]
pub struct AlignedBuffer([u8; 64]);

#[inline(never)]
#[no_mangle]
pub extern "C" fn ct_memcmp(lhs: *const u8, rhs: *const u8, len: usize) -> i32 {
    let mut acc: u8 = 0;
    for i in 0..len {
        unsafe {
            let l = core::ptr::read_volatile(lhs.add(i));
            let r = core::ptr::read_volatile(rhs.add(i));
            acc |= l ^ r;
        }
    }
    acc as i32
}

#[cfg(target_arch = "x86_64")]
#[path = "tsx_memcmp.rs"]
pub mod tsx_memcmp;

pub mod sandbox {
    pub fn sandbox() {
        println!("This is a sandbox function for testing purposes.");
    }
}

pub use crate::sandbox::sandbox;
