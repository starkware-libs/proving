pub mod bit_unpack;
pub mod div2;

#[cfg(test)]
mod test;

use crate::core::air_fn_registry::*;
use bit_unpack::*;

pub fn create_bit_unpacking_json() -> AirFnRegistry {
    AirFnRegistry::new(&BitUnpack::<4> {})
}
