use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;

use super::felt252_id_memory::*;

#[derive(Debug)]
pub struct MemVerify {
    pub memory: Felt252IdMemory,
}

/// Receives an (address, value) pair, deduces the ID, and constrains <address> to contain <value>
impl AirFn for MemVerify {
    type In = (FeltExpr, Felt252Expr);
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, (address, value): Self::In) -> Self::Out {
        let id = air_builder.mem_read_unverified(&self.memory.address_to_id, &address);
        air_builder.call(
            &MemVerifyKnownId {
                memory: self.memory.clone(),
            },
            (address, id, value),
        )
    }
}

#[derive(Debug)]
pub struct MemVerifyKnownId {
    pub memory: Felt252IdMemory,
}

/// Same as MemVerify, but receives the ID as a parameter instead of reading it from the
/// address -> ID table. More efficient than MemVerify if the ID is known (for example, as
/// a result of a previous read_unverified()).
impl AirFn for MemVerifyKnownId {
    type In = (FeltExpr, FeltExpr, Felt252Expr);
    type Out = ();

    fn call(
        &self,
        air_builder: &mut crate::core::air_fn::AirBuilder,
        (address, mut id, value): Self::In,
    ) -> Self::Out {
        air_builder.deduce(&mut id);
        air_builder.mem_verify(&self.memory.address_to_id, &address, id.clone());
        air_builder.mem_verify(&self.memory.id_to_value, &id, value);
    }
}
