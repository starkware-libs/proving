#[cfg(any(test, feature = "test"))]
use std::collections::BTreeMap;

use super::address_to_id::*;
use super::id_to_big::*;
use super::id_to_small::*;
use super::read_positive::*;
use super::read_small::*;
use crate::casm_state::*;
#[cfg(any(test, feature = "test"))]
use crate::const_expr;
#[cfg(any(test, feature = "test"))]
use crate::core::Felt;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::felt252_expr::*;
#[cfg(any(test, feature = "test"))]
use crate::core::memory::*;
use crate::core::struct_var::*;
#[cfg(any(test, feature = "test"))]
use crate::core::variables::*;
use crate::utils::*;

pub const ADDRESS_BITS: usize = 29;
pub type CasmId = VarWrapper<FeltExpr, String>;

/// Stores an address -> Felt252 mapping using three components: address -> ID,
/// ID -> small memory value and ID -> big memory value (felt252). The ID is a single M31 felt and
/// it is guaranteed that different Felt252 values have different IDs.
///
/// This representation allows to verify that two addresses contain the same value by
/// performing a lookup just in component (1), which requires deducing just three felts
/// (the two addresses, and the value ID).
#[derive(Debug, Clone, Default)]
pub struct Felt252IdMemory {
    pub(super) address_to_id: MemoryAddressToId,
    pub(super) id_to_big: MemoryIdToBig,
    #[allow(dead_code)]
    pub(super) id_to_small: MemoryIdToSmall,
}

impl Felt252IdMemory {
    #[cfg(any(test, feature = "test"))]
    pub fn new_with_data(data: Vec<(FeltExpr, Felt252Expr)>) -> Self {
        let mut value_to_id = BTreeMap::<Vec<Felt>, u32>::new();
        let mut result = Self::default();
        let mut id = 0;

        for (addr, felt252) in data {
            let limbs = felt252.to_values().expect("felt252 has no values");

            // If it is a new value, create a new ID
            if !value_to_id.contains_key(&limbs) {
                value_to_id.insert(limbs.clone(), id);
                result.id_to_big.mem_mut().set(CasmId::new(const_expr!(id), ""), felt252);
                id += 1;
            }

            // Set ID in address_to_id memory
            let felt252_id = value_to_id.get(&limbs).unwrap();
            result
                .address_to_id
                .mem_mut()
                .set(CasmAddress::new(addr.clone(), ""), CasmId::new(const_expr!(*felt252_id), ""));
        }

        result
    }

    pub fn read_unverified(
        &self,
        air_builder: &mut AirBuilder,
        address: &CasmAddress,
    ) -> (Felt252Expr, CasmId) {
        let id = air_builder.mem_read_unverified(&self.address_to_id, address);
        let value = air_builder.mem_read_unverified(&self.id_to_big, &id);
        (value, id)
    }

    pub fn read_unverified_known_id(
        &self,
        air_builder: &mut AirBuilder,
        id: &CasmId,
    ) -> Felt252Expr {
        air_builder.mem_read_unverified(&self.id_to_big, id)
    }

    pub fn read_rel_imm(&self, air_builder: &mut AirBuilder, address: CasmAddress) -> FeltExpr {
        air_builder.call(&ReadSmall { memory: self.clone() }, address).0
    }

    pub fn read_address(&self, air_builder: &mut AirBuilder, address: CasmAddress) -> CasmAddress {
        let (address_f252, _) = air_builder
            .call(&ReadPositive { memory: self.clone(), num_bits: ADDRESS_BITS }, address);

        CasmAddress::new(felt252_to_m31(address_f252, ADDRESS_BITS), "")
    }

    pub fn read_felt252(&self, air_builder: &mut AirBuilder, address: CasmAddress) -> Felt252Expr {
        air_builder.call(&ReadPositive { num_bits: 252, memory: self.clone() }, address).0
    }

    pub fn mem_verify_known_id(
        &self,
        air_builder: &mut AirBuilder,
        id: &CasmId,
        value: Felt252Expr,
    ) {
        air_builder.mem_verify(&self.id_to_big, id, value);
    }
}
