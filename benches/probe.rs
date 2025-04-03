use std::arch::x86_64::_rdtscp;
use std::fs::File;
use std::io::Write;

fn main() {
    let mut aux = 0;
    let start = unsafe { _rdtscp(&mut aux) };
    let end = unsafe { _rdtscp(&mut aux) };
    let cycles = end - start;

    let mut file = File::create("timing_results.txt").expect("Unable to create file");
    writeln!(file, "Cycles: {}", cycles).expect("Unable to write data");
}
