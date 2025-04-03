use std::process::Command;

#[test]
fn test_probe_runs_without_crash() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "probe"])
        .output()
        .expect("Failed to execute probe");
    
    assert!(output.status.success());
}

#[test]
fn test_probe_output_format() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "probe"])
        .output()
        .expect("Failed to execute probe");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check for expected output sections
    assert!(stdout.contains("Running memory comparison probe..."));
    assert!(stdout.contains("Buffer size:"));
    assert!(stdout.contains("Iterations:"));
    assert!(stdout.contains("Cycle delta:"));
    assert!(stdout.contains("Branch misses:"));
    assert!(stdout.contains("Cache references:"));
    assert!(stdout.contains("Cache misses:"));
    assert!(stdout.contains("Cache miss rate:"));
}

#[test]
fn test_probe_performance_sanity() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "probe"])
        .output()
        .expect("Failed to execute probe");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Basic sanity checks on the performance metrics
    for line in stdout.lines() {
        if line.contains("Cycle delta:") {
            let delta: i64 = line.split(':')
                .nth(1)
                .unwrap()
                .trim()
                .split_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            
            // Delta should be reasonable (not extreme)
            assert!(delta > -10000 && delta < 10000);
        }
        
        if line.contains("Cache miss rate:") {
            let rate: f64 = line.split(':')
                .nth(1)
                .unwrap()
                .trim()
                .split('%')
                .next()
                .unwrap()
                .parse()
                .unwrap();
            
            // Miss rate should be between 0% and 100%
            assert!(rate >= 0.0 && rate <= 100.0);
        }
    }
}
