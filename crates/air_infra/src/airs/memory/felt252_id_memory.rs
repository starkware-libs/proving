#[cfg(test)]
use std::collections::BTreeMap;

use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

#[cfg(test)]
use crate::core::variables::*;

#[cfg(test)]
use crate::core::Felt;

#[cfg(test)]
use crate::expr;

#[cfg(test)]
use crate::const_expr;

/// Stores an address -> Felt252 mapping using two components: 1. address -> ID table and
/// 2. ID -> Felt252 table. The ID is a single M31 felt and it is guaranteed that different
/// Felt252 values have different IDs.
///
/// This representation allows to verify that two addresses contain the same value by
/// performing a lookup just in component (1), which requires deducing just three felts
/// (the two addresses, and the value ID).
#[derive(Debug, Clone, Default)]
pub struct Felt252IdMemory {
    pub(super) address_to_id: Memory<FeltExpr, FeltExpr>,
    pub(super) id_to_value: Memory<FeltExpr, Felt252Expr>,
}

impl Felt252IdMemory {
    #[cfg(test)]
    pub fn new_with_data(data: Vec<(FeltExpr, Felt252Expr)>) -> Self {
        let mut value_to_id = BTreeMap::<Vec<Felt>, u32>::new();
        let result = Self::default();
        let mut id = 0;

        for (addr, felt252) in data {
            let limbs = felt252.to_values();

            // If it is a new value, create a new ID
            if !value_to_id.contains_key(&limbs) {
                value_to_id.insert(limbs.clone(), id);
                result.id_to_value.set(const_expr!(id), felt252);
                id += 1;
            }

            // Set ID in address_to_id memory
            let felt252_id = value_to_id.get(&limbs).unwrap();
            result.address_to_id.set(addr, expr!("id", *felt252_id));
        }

        result
    }

    pub fn read(
        &self,
        air_builder: &mut AirBuilder,
        address: &FeltExpr,
    ) -> (Felt252Expr, FeltExpr) {
        let id = air_builder.mem_read(&self.address_to_id, address);
        let value = air_builder.mem_read(&self.id_to_value, &id);
        (value, id)
    }
}
