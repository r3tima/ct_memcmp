#![no_std]

use core::num::Wrapping;
pub const SANDBOX_MEMORY_LIMIT: usize = 1 << 20;
pub const ALIGNMENT_REQUIREMENT: usize = 64;

#[cfg(target_feature = "sse2")]
#[inline(always)]
pub fn lfence() {
    unsafe {
        core::arch::asm!("lfence", options(nostack, preserves_flags));
    }
}

#[inline]
pub fn check_address_bounds(addr: usize, size: usize) -> bool {
    let end_addr = Wrapping(addr) + Wrapping(size);
    addr <= end_addr.0 && end_addr.0 <= SANDBOX_MEMORY_LIMIT
}

#[inline]
pub fn validate_alignment(addr: usize) -> bool {
    addr % ALIGNMENT_REQUIREMENT == 0
}

#[inline]
pub fn sanitize_data(data: u64) -> u64 {
    data & 0x0000_FFFF_FFFF_FFFF
}

pub fn data_sandboxing(addr: usize, size: usize, data: u64) -> Result<u64, &'static str> {
    if !check_address_bounds(addr, size) {
        return Err("Memory access out of sandbox bounds");
    }

    if !validate_alignment(addr) {
        return Err("Memory address not properly aligned");
    }

    #[cfg(target_feature = "sse2")]
    lfence();

    let sanitized_data = sanitize_data(data);

    Ok(sanitized_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_bounds() {
        assert!(check_address_bounds(0, 1024));
        assert!(!check_address_bounds(SANDBOX_MEMORY_LIMIT - 10, 100));
    }

    #[test]
    fn test_alignment() {
        assert!(validate_alignment(64));
        assert!(validate_alignment(128));
        assert!(!validate_alignment(63));
    }

    #[test]
    fn test_data_sanitization() {
        assert_eq!(sanitize_data(0xFFFF_FFFF_FFFF_FFFF), 0x0000_FFFF_FFFF_FFFF);
        assert_eq!(sanitize_data(0x0000_1234_5678_9ABC), 0x0000_1234_5678_9ABC);
    }
}
