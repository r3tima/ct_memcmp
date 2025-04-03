#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{_mm_clflush, _mm_lfence};
use std::time::Instant;
use std::arch::asm;

const CACHE_HIT_THRESHOLD: u64 = 80; 
const ORACLE_SIZE: usize = 256 * 4096; 

#[inline(never)]
#[no_mangle]
pub unsafe fn tsx_memcmp(
    secret: *const u8, 
    input: *const u8, 
    len: usize,
    oracle: *mut u8,
    results: *mut u64
) {
    for i in 0..ORACLE_SIZE {
        _mm_clflush(oracle.add(i));
    }
    _mm_lfence();

    let mut status: i32;
    asm!("xbegin 2f",
         "mov {0}, 0",
         "jmp 3f",
         "2:",
         "mov {0}, 1",
         "3:",
         out(reg) status,
         options(nostack)
    );

    if status == 0 {
        for i in 0..len {
            let s = *secret.add(i);
            let c = *input.add(i);
            if s != c {
                let _ = *oracle.add(s as usize * 4096);
            }
        }
        asm!("xend", options(nostack));
    }

    for i in 0..256 {
        let addr = oracle.add(i * 4096);
        let start = Instant::now();
        let _ = *addr;
        let delta = start.elapsed().as_nanos() as u64;
        *results.add(i) = delta;
    }
}
