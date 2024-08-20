use crate::{airs::casm::common::CasmAddress, core::air_fn::*};

use super::felt252_id_memory::*;

#[derive(Debug)]
pub struct MemVerifyEqual {
    pub memory: Felt252IdMemory,
}

/// Verifies that the values in the given addresses are equal by deducing just the value ID.
impl AirFn for MemVerifyEqual {
    type In = [CasmAddress; 2];
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, [addr1, addr2]: Self::In) -> Self::Out {
        let mut id = air_builder.mem_read_unverified(&self.memory.address_to_id, &addr1);
        air_builder.deduce(&mut id);
        air_builder.mem_verify(&self.memory.address_to_id, &addr1, id.clone());
        air_builder.mem_verify(&self.memory.address_to_id, &addr2, id);
    }
}
