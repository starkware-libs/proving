use inst_def::InstDef;

use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;

use super::felt252_id_memory::*;

#[derive(Debug, InstDef)]
pub struct MemVerify {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

/// Receives an (address, value) pair, deduces the ID, and constrains <address> to contain <value>
impl AirFn for MemVerify {
    type In = (FeltExpr, Felt252Expr);
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, (address, value): Self::In) -> Self::Out {
        let mut id = air_builder.mem_read_unverified(&self.memory.address_to_id, &address);
        air_builder.deduce(&mut id);
        air_builder.mem_verify(&self.memory.address_to_id, &address, id.clone());
        air_builder.mem_verify(&self.memory.id_to_value, &id, value);
    }
}
