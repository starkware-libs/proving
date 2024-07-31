#[cfg(test)]
use std::collections::BTreeMap;

use indexmap::IndexMap;

use crate::airs::casm::common::*;
use crate::airs::range_check::RangeCheck;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::FELT252_BITS_PER_WORD;
use crate::core::variables::AirVar;

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
    address_to_id: Memory<FeltExpr, FeltExpr>,
    id_to_value: Memory<FeltExpr, Felt252Expr>,
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
}

#[derive(Debug)]
pub struct ReadPositive {
    pub num_bits: usize,
    pub memory: Felt252IdMemory,
}

/// Read a Felt252 in the range [0,2**num_bits - 1] from the memory
impl AirFn for ReadPositive {
    type In = CasmAddress;
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, address: Self::In) -> Self::Out {
        // Read the id and deduce it as-is
        let mut id = air_builder.get_from_memory(&self.memory.address_to_id, &address);
        air_builder.deduce(&mut id);
        air_builder.set_in_memory(&self.memory.address_to_id, address, id.clone());

        // Prepare for value deduction
        let mut value = air_builder.get_from_memory(&self.memory.id_to_value, &id);
        let mut value_felts = value.as_felts_mut();
        let num_nonzero_limbs = self.num_bits.div_ceil(FELT252_BITS_PER_WORD);
        let bits_in_ms_limb = self.num_bits % FELT252_BITS_PER_WORD;

        // Deduce the nonzero limbs
        for limb in value_felts.iter_mut().take(num_nonzero_limbs) {
            air_builder.deduce(limb);
        }

        // If required - range-check the most significant limb
        if bits_in_ms_limb != 0 {
            air_builder.lookup_call(
                &RangeCheck {
                    bits: bits_in_ms_limb as u16,
                },
                value
                    .as_felts()
                    .into_iter()
                    .nth(num_nonzero_limbs - 1)
                    .expect("The value should have enough limbs"),
            );
        }

        let expected_value_in_memory = Felt252Expr::from(
            value
                .as_felts()
                .into_iter()
                .take(num_nonzero_limbs)
                .collect::<Vec<_>>(),
        );

        // Verify that the value in memory is the nonzero limbs we deduced, padded on
        // the left with zeros.
        air_builder.set_in_memory(
            &self.memory.id_to_value,
            id,
            expected_value_in_memory.clone(),
        );

        expected_value_in_memory
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        [("num_bits".to_string(), self.num_bits.to_string())].into()
    }
}
