use libc::{mmap, mprotect, PROT_NONE, PROT_READ, PROT_WRITE, MAP_ANONYMOUS, MAP_PRIVATE};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

static FAULT_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct FaultInjector {
    base: *mut u8,
    size: usize,
    fault_offset: usize,
}

impl FaultInjector {
    pub fn new(size: usize, fault_offset: usize) -> Option<Self> {
        unsafe {

            let base = mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_ANONYMOUS | MAP_PRIVATE,
                -1,
                0
            ) as *mut u8;
            
            if base.is_null() {
                return None;
            }
            
            // Set up guard page
            let guard_addr = base.add(fault_offset);
            mprotect(
                guard_addr as *mut libc::c_void,
                4096,
                PROT_NONE
            );
            
            Some(Self { base, size, fault_offset })
        }
    }
    
    pub fn inject_faults(&self, func: unsafe extern fn(*const u8, *const u8, usize) -> i32) -> i32 {
        unsafe {
            libc::signal(libc::SIGSEGV, handle_sigsegv as libc::sighandler_t);
            
            let result = func(
                self.base,
                self.base.add(self.size / 2),
                self.size / 2
            );
            
            libc::signal(libc::SIGSEGV, libc::SIG_DFL);
            result
        }
    }
}

extern "C" fn handle_sigsegv(_sig: i32) {
    FAULT_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    let ctx = unsafe { *libc::__ucontext_get() };
    let fault_addr = ctx.uc_mcontext.fault_address as usize;
    
    unsafe {
        let mut altstack = std::mem::zeroed();
        libc::sigaltstack(ptr::null(), &mut altstack);
        
        if altstack.ss_flags & libc::SA_ONSTACK != 0 {
            libc::sigaltstack(ptr::null_mut(), ptr::null_mut());
        }
    }
}
