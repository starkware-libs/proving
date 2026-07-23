pub const EXTENSION_DEGREE: usize = 4;

pub mod blake;
pub mod circuit;
pub mod context;
pub mod extract_bits;
pub mod finalize_constants;
pub mod ivalue;
pub mod ops;
pub mod simd;
pub mod stats;
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
pub mod utils;
pub mod wrappers;
