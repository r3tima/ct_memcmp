use core::arch::x86_64::_rdtsc;
use memcopy::ct_memcmp;

// Performance counter constants
const PERF_TYPE_HARDWARE: u32 = 0;
const PERF_COUNT_HW_CACHE_REFERENCES: u64 = 0;
const PERF_COUNT_HW_CACHE_MISSES: u64 = 1;
const PERF_COUNT_HW_BRANCH_MISSES: u64 = 2;

#[repr(C)]
struct perf_event_attr {
    type_: u32,
    size: u32,
    config: u64,
    sample_period: u64,
    sample_type: u64,
    read_format: u64,
    flags: u64,
    
    // Bitfield values packed into flags
    disabled: u32,         // off by default
    inherit: u32,          // children inherit it
    pinned: u32,           // must always be on PMU
    exclusive: u32,        // only group on PMU
    exclude_user: u32,     // don't count user
    exclude_kernel: u32,   // don't count kernel
    exclude_hv: u32,       // don't count hypervisor
    exclude_idle: u32,     // don't count when idle
    
    wakeup_events: u32,
    bp_type: u32,
    bp_addr: u64,
    bp_len: u64,
    
    branch_sample_type: u64,
    sample_regs_user: u64,
    sample_stack_user: u32,
    clockid: i32,
    sample_regs_intr: u64,
    aux_watermark: u32,
    sample_max_stack: u16,
    __reserved_2: u16,
}

use std::{
    alloc::{alloc, Layout},
    mem, ptr,
    thread::sleep,
    time::Duration,
};

const BUFFER_SIZE: usize = 64;
const ITERATIONS: usize = 1000;
const WARMUP_MS: u64 = 500;

#[derive(Debug)]
struct PerfCounters {
    branch_misses: i64,
    cache_refs: i64,
    cache_misses: i64,
}

struct PerfEvents {
    branch_fd: i32,
    cache_ref_fd: i32,
    cache_miss_fd: i32,
}

fn setup_perf_event(_event_type: u32, config: u64) -> i32 {
    let mut attr: perf_event_attr = unsafe { mem::zeroed() };
    attr.type_ = PERF_TYPE_HARDWARE;
    attr.size = mem::size_of::<perf_event_attr>() as u32;
    attr.config = config;
    attr.disabled = 0;
    attr.exclude_kernel = 1;
    attr.exclude_hv = 1;
    attr.read_format = 0x0000000000000001; // PERF_FORMAT_TOTAL_TIME_ENABLED

    unsafe {
        libc::syscall(
            libc::SYS_perf_event_open,
            &mut attr as *mut _,
            0i32,  // current process
            -1i32, // all CPUs
            -1i32, // no group
            0u32,  // no flags
        ) as i32
    }
}

fn setup_perf_events() -> PerfEvents {
    PerfEvents {
        branch_fd: setup_perf_event(PERF_TYPE_HARDWARE, PERF_COUNT_HW_BRANCH_MISSES as u64),
        cache_ref_fd: setup_perf_event(PERF_TYPE_HARDWARE, PERF_COUNT_HW_CACHE_REFERENCES as u64),
        cache_miss_fd: setup_perf_event(PERF_TYPE_HARDWARE, PERF_COUNT_HW_CACHE_MISSES as u64),
    }
}

fn read_counter(fd: i32) -> i64 {
    let mut value: i64 = 0;
    unsafe {
        libc::read(fd, &mut value as *mut i64 as *mut _, 8);
    }
    value
}

fn allocate_buffers() -> (*mut u8, *mut u8) {
    let layout = Layout::from_size_align(BUFFER_SIZE, 64).unwrap();
    unsafe {
        let hot = alloc(layout);
        let cold = alloc(layout);
        
        // Initialize buffers
        ptr::write_bytes(hot, 0xAA, BUFFER_SIZE);
        ptr::write_bytes(cold, 0xAA, BUFFER_SIZE);
        
        (hot, cold)
    }
}

fn measure_memcmp_delta(hot: *const u8, cold: *const u8) -> (u64, PerfCounters) {
    let events = setup_perf_events();
    
    // Warm up the hot buffer
    for _ in 0..ITERATIONS {

            ct_memcmp(hot, hot, BUFFER_SIZE);

    }
    
    // Sleep to allow for thermal throttling
    sleep(Duration::from_millis(WARMUP_MS));
    
    // Measure hot access
    let start_hot = unsafe { _rdtsc() };
    unsafe {
        ct_memcmp(hot, hot, BUFFER_SIZE);
    }
    let end_hot = unsafe { _rdtsc() };
    
    // Measure cold access
    let start_cold = unsafe { _rdtsc() };
    unsafe {
        ct_memcmp(cold, cold, BUFFER_SIZE);
    }
    let end_cold = unsafe { _rdtsc() };
    
    let counters = PerfCounters {
        branch_misses: read_counter(events.branch_fd),
        cache_refs: read_counter(events.cache_ref_fd),
        cache_misses: read_counter(events.cache_miss_fd),
    };
    
    // Calculate deltas using wrapping arithmetic
    let hot_delta = end_hot.wrapping_sub(start_hot);
    let cold_delta = end_cold.wrapping_sub(start_cold);
    
    // Convert to signed i32 to handle potential negative differences
    let hot_cycles = (hot_delta & 0xFFFFFFFF) as i32;
    let cold_cycles = (cold_delta & 0xFFFFFFFF) as i32;
    let delta = cold_cycles - hot_cycles;
    
    (delta as u64 & 0xFFFFFFFF, counters)
}

fn main() {
    let (hot, cold) = allocate_buffers();
    
    println!("Running memory comparison probe...");
    println!("Buffer size: {} bytes", BUFFER_SIZE);
    println!("Iterations: {}", ITERATIONS);
    println!("Warmup time: {}ms", WARMUP_MS);
    println!();
    
    for i in 0..5 {
        let (delta, counters) = measure_memcmp_delta(hot, cold);
        println!("Run {}:", i + 1);
        println!("  Cycle delta: {} cycles", delta);
        println!("  Branch misses: {}", counters.branch_misses);
        println!("  Cache references: {}", counters.cache_refs);
        println!("  Cache misses: {}", counters.cache_misses);
        println!("  Cache miss rate: {:.2}%", 
            (counters.cache_misses as f64 / counters.cache_refs as f64) * 100.0);
        println!();
        
        // Allow system to stabilize between runs
        sleep(Duration::from_millis(100));
    }
    
    // Clean up
    unsafe {
        let layout = Layout::from_size_align(BUFFER_SIZE, 64).unwrap();
        std::alloc::dealloc(hot as *mut u8, layout);
        std::alloc::dealloc(cold as *mut u8, layout);
    }
}
