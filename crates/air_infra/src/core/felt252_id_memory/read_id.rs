use serde::Serialize;

use super::memory::*;
use crate::airs::casm::casm_state::*;
//  Macros
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

#[derive(Debug, Serialize)]
pub struct ReadId {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

// Reads and verifies the ID of the value at the given address in the memory.
impl AirFn for ReadId {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), address: Self::In) -> Self::Out {
        let mut id = air_builder.mem_read_unverified(&self.memory.address_to_id, &address);

        // Deduce the ID as-is
        air_builder.deduce(
            &mut id,
            &address
                .extra_info
                .clone()
                .map(|s| format!("{}_id", s))
                .unwrap_or("id".to_string()),
        );

        air_builder.mem_verify(&self.memory.address_to_id, &address, id.clone());

        id
    }
}
