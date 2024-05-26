pub mod fib;
pub mod fib_step;

#[cfg(test)]
mod test;

use crate::core::air_fn_registry::*;
use fib::*;

pub fn create_fibonacci_json() {
    let registry = AirFnRegistry::new(&Fib { claim_index: 6 });
    registry.dump_to_file("fibonacci/air.json");
}
