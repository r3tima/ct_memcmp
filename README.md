# ct_memcmp

ct_memcmp is a minimal, FFI-safe, cryptographically hygienic constant time comparator implemented in pure rust with optional inline x86_64 assembly and speculative execution mitigations. the core design enforces deterministic, branchless execution across arbitrarily aligned memory regions using bitwise accumulator folding to eliminate data-dependent control flow. all logic is encoded within a strictly monotonic instruction stream free of conditional branches or short-circuiting logic, ensuring uniform cacheline traversal and mitigating classic microarchitectural timing disclosure vectors.

internally, the comparator emits a data-oblivious diff stream by computing the bitwise xor delta per byte and collapsing the result via an OR-reduction accumulator. each memory dereference is wrapped in volatile semantics to prevent the optimizer from reordering, merging, or short-circuiting accesses based on static equivalence heuristics. inputs may be optionally aligned to 64-byte boundaries to enforce L1d cacheline uniformity and simulate cache probe scenarios under adversarial buffer layout.

it includes optional architectural hardening for speculative execution leaks. under the sse2 feature gate, execution emits an lfence barrier post-load to serialize dispatch and inhibit speculative branching past load gates. the implementation supports rdtscp delta measurement as an instrumentation hook to detect cycle-based variance under kvm, qemu, or bare metal perf event contexts. performance deltas are extracted via a dedicated probe binary that invokes ct_memcmp() across thermally isolated hot and cold buffers, then dumps the timing histogram alongside perf_event_open counters for raw branch misses, dTLB lookups, and LLC ref/miss telemetry.

integration is possible via the exported #[no_mangle] C ABI function signature which accepts raw u8 pointers and buffer length. all public interfaces are marked #[inline(never)] and compiled under opt-level=z with lto and frame pointers enabled to preserve instruction shape and enforce maximal observability under dynamic analysis. the build config pins target-cpu=native and enforces -z now via linker args to eliminate lazy plt resolution and reduce variance in cold start execution flows.

fuzz instrumentation is provided via a harness that randomizes buffer values, length fields, and allocation patterns across page boundaries to surface timing skew under afl or honggfuzz. cache footprint and branch consistency are validated across thousands of trials and diffed against randomized patterns. delta amplification is optionally induced via rdtscp tightloop profiling to exaggerate per-byte access deltas in the presence of speculative predictors.

### constant time primitives

the core implementation provides constant time memory comparison through carefully crafted assembly sequences:

```rust
pub fn ct_memcmp(a: *const u8, b: *const u8, len: usize) -> i32
```

### performance probing

includes a probe binary for analysing memory access patterns:

```rust
fn measure_memcmp_delta(hot: *const u8, cold: *const u8) -> (u64, PerfCounters)
```

## usage

### basic memory operations

```rust
use memcopy::ct_memcmp;

let result = ct_memcmp(buf1.as_ptr(), buf2.as_ptr(), 64);
```

### performance analysis

```bash
cargo run --bin probe
# or  
sudo cargo run --bin probe
```
