pub mod fib;
pub mod fib_step;
pub mod narrow_fib;
pub mod wide_fib;

#[cfg(test)]
mod test;

use crate::core::air_fn_registry::*;
use fib::*;

pub fn create_fibonacci_json() -> AirFnRegistry {
    AirFnRegistry::new(&Fib { claim_index: 6 })
}
