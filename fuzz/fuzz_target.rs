#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 2 {
        let mid = data.len() / 2;
        let (lhs, rhs) = data.split_at(mid);
        let _ = crate::ct_memcmp(lhs.as_ptr(), rhs.as_ptr(), lhs.len().min(rhs.len()));
    }
});
