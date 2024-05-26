pub mod add32;
#[cfg(test)]
mod add32_test;

use crate::core::air_fn_registry::*;
use add32::*;

pub fn create_add32_json() {
    let registry = AirFnRegistry::new(&Add32 {});
    registry.dump_to_file("blake/add32.json");
}
