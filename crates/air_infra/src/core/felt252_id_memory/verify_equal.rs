use inst_def::InstDef;

use super::memory::*;
use crate::airs::casm::casm_state::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

#[derive(Debug, InstDef)]
pub struct MemVerifyEqual {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

/// Verifies that the values in the given addresses are equal by deducing just the value ID.
impl AirFn for MemVerifyEqual {
    type In = [CasmAddress; 2];
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, [addr1, addr2]: Self::In) -> Self::Out {
        let mut id = air_builder.mem_read_unverified(&self.memory.address_to_id, &addr1.value);
        air_builder.deduce(
            &mut id,
            &addr1
                .desc
                .map(|s| format!("{}_id", s))
                .unwrap_or("id".to_string()),
        );
        air_builder.mem_verify(&self.memory.address_to_id, &addr1.value, id.clone());
        air_builder.mem_verify(&self.memory.address_to_id, &addr2.value, id);
    }
}

#[derive(Debug, InstDef)]
pub struct MemCondVerifyEqualKnownId {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

/// Same as MemVerifyEqual, but receives a condition so that the values are verified to be equal
/// only when the given condition is met. The condition is created after reading one of the values
/// with ReadSmall for example, so there is no need to read, deduce or verifiy its ID.
impl AirFn for MemCondVerifyEqualKnownId {
    type In = (CasmAddress, FeltExpr, FeltExpr);
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, (addr1, id2, cond): Self::In) -> Self::Out {
        let mut id1 = air_builder.mem_read_unverified(&self.memory.address_to_id, &addr1.value);
        air_builder.deduce(
            &mut id1,
            &addr1
                .desc
                .map(|s| format!("{}_id", s))
                .unwrap_or("id".to_string()),
        );
        air_builder.mem_verify(&self.memory.address_to_id, &addr1.value, id1.clone());

        air_builder.constrain(
            (id1 - id2) * cond,
            "The two ids are equal if the condition is met",
        );
    }
}
