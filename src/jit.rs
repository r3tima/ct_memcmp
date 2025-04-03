use std::arch::x86_64::{_mm_clflush, _mm_lfence, _mm_mfence};
use std::mem;
use libc::{mmap, mprotect, munmap, PROT_READ, PROT_WRITE, PROT_EXEC, MAP_ANONYMOUS, MAP_PRIVATE};
use rand::Rng;

const PAGE_SIZE: usize = 4096;

pub struct JitBuffer {
    ptr: *mut u8,
    size: usize,
}

impl JitBuffer {
    pub fn new(size: usize) -> Option<Self> {
        unsafe {
            let ptr = mmap(
                std::ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE | PROT_EXEC,
                MAP_ANONYMOUS | MAP_PRIVATE,
                -1,
                0
            ) as *mut u8;
            
            if ptr.is_null() {
                None
            } else {
                Some(Self { ptr, size })
            }
        }
    }
    
    pub fn write_instructions(&mut self, offset: usize, data: &[u8]) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                self.ptr.add(offset),
                data.len()
            );
            _mm_clflush(self.ptr.add(offset));
        }
    }
    
    pub fn make_executable(&self) {
        unsafe {
            mprotect(
                self.ptr as *mut libc::c_void,
                self.size,
                PROT_READ | PROT_EXEC
            );
            _mm_mfence();
        }
    }
}

impl Drop for JitBuffer {
    fn drop(&mut self) {
        unsafe {
            munmap(self.ptr as *mut libc::c_void, self.size);
        }
    }
}

#[derive(Default)]
struct CodeGenerator {
    buffer: Vec<u8>,
    reg_map: [u8; 8],
}

impl CodeGenerator {
    fn new() -> Self {
        let mut gen = Self::default();
        gen.randomize_registers();
        gen
    }
    
    fn randomize_registers(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut regs = [0u8, 1u8, 2u8, 3u8];
        rng.shuffle(&mut regs);
        self.reg_map = regs;
    }
    
    fn emit_mov(&mut self, dst: u8, src: u8) {
        self.buffer.extend_from_slice(&[0x48, 0x8B, 0xC0 | (dst << 3) | src]);
    }
    
    fn emit_xor(&mut self, dst: u8, src: u8) {
        self.buffer.extend_from_slice(&[0x48, 0x31, 0xC0 | (dst << 3) | src]);
    }
    
    fn emit_or(&mut self, dst: u8, src: u8) {
        self.buffer.extend_from_slice(&[0x48, 0x09, 0xC0 | (dst << 3) | src]);
    }
    
    fn generate_ct_memcmp(&mut self, size: usize) {
        self.emit_mov(self.reg_map[0], 0); 
        self.emit_mov(self.reg_map[1], 1); 
        
        let loop_start = self.buffer.len();
        
        self.emit_mov(self.reg_map[2], 0); // lhs
        self.emit_mov(self.reg_map[3], 1); // rhs
        
        self.emit_xor(self.reg_map[2], self.reg_map[3]);
        self.emit_or(self.reg_map[1], self.reg_map[2]);
    
        self.emit_mov(self.reg_map[0], self.reg_map[0] + 1);
        
        self.buffer.extend_from_slice(&[
            0x48, 0x81, 0xF8,
            (size as u32).to_le_bytes()[0],
            (size as u32).to_le_bytes()[1],
            (size as u32).to_le_bytes()[2],
            (size as u32).to_le_bytes()[3],
            0x72, 
            (loop_start as i8 - self.buffer.len() as i8 - 1) as u8
        ]);
        
        self.buffer.push(0xC3); 
    }
}

pub fn compile_ct_memcmp(size: usize) -> JitBuffer {
    let mut gen = CodeGenerator::new();
    gen.generate_ct_memcmp(size);
    
    let mut jit = JitBuffer::new(gen.buffer.len()).unwrap();
    jit.write_instructions(0, &gen.buffer);
    jit.make_executable();
    
    jit
}

#[cfg(target_arch = "aarch64")]
mod mte {
    use super::*;
    
    pub fn generate_mte_memcmp(size: usize) -> JitBuffer {
        let mut gen = CodeGenerator::new();
        
        gen.buffer.extend_from_slice(&[
            0xE8, 0x07, 0x00, 0x58,
            0xE8, 0x03, 0x00, 0x91, 
            0x09, 0x01, 0x40, 0xF9, 
        ]);
        
        gen.generate_ct_memcmp(size);
        
        let mut jit = JitBuffer::new(gen.buffer.len()).unwrap();
        jit.write_instructions(0, &gen.buffer);
        jit.make_executable();
        
        jit
    }
}

#[cfg(target_arch = "aarch64")]
pub use mte::generate_mte_memcmp;
