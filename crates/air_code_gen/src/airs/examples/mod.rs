pub mod bit_unpack;
pub mod fibonacci;
pub mod narrow_fibonacci;
#[cfg(test)]
pub mod test_utils;
pub mod wide_fib;

use narrow_fibonacci::simd_trace::NarrowFib_1ddf31c88316e62fSimdTraceGenerator;
use narrow_fibonacci::trace::NarrowFib_1ddf31c88316e62fCpuTraceGenerator;
