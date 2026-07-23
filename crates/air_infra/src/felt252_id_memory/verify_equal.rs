use serde::Serialize;

use super::memory::*;
use crate::casm_state::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::felt252_id_memory::read_id::*;

#[derive(Debug, Serialize)]
pub struct MemVerifyEqual {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

/// Verifies that the values in the given addresses are equal by deducing just the value ID.
impl AirFn for MemVerifyEqual {
    type ExtIn = ();
    type In = [CasmAddress; 2];
    type Out = ();

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("address1".to_string()), Some("address2".to_string())])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), [addr1, addr2]: Self::In) -> Self::Out {
        let id = air_builder.call(&ReadId { memory: self.memory.clone() }, addr1.clone());
        air_builder.mem_verify(&self.memory.address_to_id, &addr2, id);
    }
}

#[derive(Debug, Serialize)]
pub struct MemCondVerifyEqualKnownId {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

/// Same as MemVerifyEqual, but receives a condition so that the values are verified to be equal
/// only when the given condition is met. The condition is created after reading one of the values
/// with ReadSmall for example, so there is no need to read, deduce or verify its ID.
impl AirFn for MemCondVerifyEqualKnownId {
    type ExtIn = ();
    type In = (CasmAddress, CasmId, FeltExpr);
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), (addr1, id2, cond): Self::In) -> Self::Out {
        let id1 = air_builder.call(&ReadId { memory: self.memory.clone() }, addr1.clone());

        air_builder
            .constrain((id1.var - id2.var) * cond, "The two ids are equal if the condition is met");
    }
}
