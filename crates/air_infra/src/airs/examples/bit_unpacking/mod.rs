pub mod bit_unpack;
pub mod div2;

#[cfg(test)]
mod test;

use crate::core::air_fn_registry::*;
use bit_unpack::*;

pub fn create_bit_unpacking_json() {
    let registry = AirFnRegistry::new(&BitUnpack { n_bits: 4 });
    registry.dump_to_file("airs/examples/bit_unpacking/air.json");
}
