#![cfg(test)]
#![no_std]

use core::arch::x86_64::_rdtscp;

#[test]
fn test_ct_memcmp() {
    let a = [1u8, 2, 3, 4];
    let b = [1u8, 2, 3, 4];
    let c = [1u8, 2, 3, 5];

    assert_eq!(crate::ct_memcmp(a.as_ptr(), b.as_ptr(), a.len()), 0);
    assert_ne!(crate::ct_memcmp(a.as_ptr(), c.as_ptr(), a.len()), 0);
}

#[test]
fn test_rdtscp() {
    let mut aux = 0;
    let start = unsafe { _rdtscp(&mut aux) };
    crate::ct_memcmp([0u8; 64].as_ptr(), [0u8; 64].as_ptr(), 64);
    let end = unsafe { _rdtscp(&mut aux) };
    let cycles = end - start;
    assert!(cycles > 0);
}
