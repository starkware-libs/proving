use inst_def::InstDef;

use super::memory::*;
use crate::airs::casm::casm_state::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;

#[derive(Debug, InstDef)]
pub struct MemVerify {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

/// Receives an (address, value) pair, deduces the ID, and constrains <address> to contain <value>
impl AirFn for MemVerify {
    type ExtIn = ();
    type In = (CasmAddress, Felt252Expr);
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), (address, value): Self::In) -> Self::Out {
        let mut id = air_builder.mem_read_unverified(&self.memory.address_to_id, &address);
        air_builder.deduce(
            &mut id,
            &address
                .desc
                .clone()
                .map(|s| format!("{}_id", s))
                .unwrap_or("id".to_string()),
        );
        air_builder.mem_verify(&self.memory.address_to_id, &address, id.clone());
        air_builder.mem_verify(&self.memory.id_to_value, &id, value);
    }
}

/// Receives an array of addresses and a value. Verifies that all the addresses contain this value.
#[derive(Debug, InstDef)]
pub struct MemVerifyAll<const N: usize> {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl<const N: usize> AirFn for MemVerifyAll<N> {
    type ExtIn = ();
    type In = ([CasmAddress; N], Felt252Expr);
    type Out = ();

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: Self::ExtIn,
        (addresses, value): Self::In,
    ) -> Self::Out {
        let mut id = air_builder.mem_read_unverified(&self.memory.address_to_id, &addresses[0]);
        air_builder.deduce(
            &mut id,
            &addresses[0]
                .desc
                .clone()
                .map(|s| format!("{}_id", s))
                .unwrap_or("id".to_string()),
        );
        air_builder.mem_verify(&self.memory.address_to_id, &addresses[0], id.clone());
        air_builder.mem_verify(&self.memory.id_to_value, &id, value);

        for address in addresses.iter().skip(1) {
            air_builder.mem_verify(&self.memory.address_to_id, address, id.clone());
        }
    }
}
