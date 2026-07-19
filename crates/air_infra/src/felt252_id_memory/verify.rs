use serde::Serialize;

use super::memory::*;
use crate::casm_state::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::felt252_id_memory::read_id::*;

#[derive(Debug, Serialize)]
pub struct MemVerify {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

/// Receives an (address, value) pair, deduces the ID, and constrains <address> to contain <value>
impl AirFn for MemVerify {
    type ExtIn = ();
    type In = (CasmAddress, Felt252Expr);
    type Out = ();

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("address".to_string()), Some("value".to_string())])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), (address, value): Self::In) -> Self::Out {
        let id = air_builder.call(&ReadId { memory: self.memory.clone() }, address.clone());

        air_builder.mem_verify(&self.memory.id_to_big, &id, value);
    }
}

/// Receives an array of addresses and a value. Verifies that all the addresses contain this value.
#[derive(Debug, Serialize)]
pub struct MemVerifyAll<const N: usize> {
    n: usize,
    #[serde(skip)]
    memory: Felt252IdMemory,
}

impl<const N: usize> MemVerifyAll<N> {
    pub fn new(memory: Felt252IdMemory) -> Self {
        Self { n: N, memory }
    }
}

impl<const N: usize> AirFn for MemVerifyAll<N> {
    type ExtIn = ();
    type In = ([CasmAddress; N], Felt252Expr);
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), (addresses, value): Self::In) -> Self::Out {
        let id = air_builder.call(&ReadId { memory: self.memory.clone() }, addresses[0].clone());
        air_builder.mem_verify(&self.memory.id_to_big, &id, value);

        for address in addresses.iter().skip(1) {
            air_builder.mem_verify(&self.memory.address_to_id, address, id.clone());
        }
    }
}
