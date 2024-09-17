#[cfg(test)]
use std::collections::BTreeMap;

use crate::airs::casm::common::*;
use crate::core::air_fn::*;
use crate::core::expressions::bool_expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

use crate::core::prover_types::*;
#[cfg(test)]
use crate::core::variables::*;

#[cfg(test)]
use crate::core::Felt;

use crate::const_expr;

use super::felt252_id_memory_read_positive::*;
use super::felt252_id_memory_read_small::*;

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
            let limbs = felt252.to_values().expect("felt252 has no values");

            // If it is a new value, create a new ID
            if !value_to_id.contains_key(&limbs) {
                value_to_id.insert(limbs.clone(), id);
                result.id_to_value.set(const_expr!(id), felt252);
                id += 1;
            }

            // Set ID in address_to_id memory
            let felt252_id = value_to_id.get(&limbs).unwrap();
            result.address_to_id.set(addr, const_expr!(*felt252_id));
        }

        result
    }

    pub fn read_unverified(
        &self,
        air_builder: &mut AirBuilder,
        address: &FeltExpr,
    ) -> (Felt252Expr, FeltExpr) {
        let id = air_builder.mem_read_unverified(&self.address_to_id, address);
        let value = air_builder.mem_read_unverified(&self.id_to_value, &id);
        (value, id)
    }

    // Receives a Felt252 and its sign bits, and returns it as a relative immediate felt.
    pub fn small_to_rel_imm(
        low_limbs: [FeltExpr; LIMBS_IN_M31],
        msb: BoolExpr,
        mid_limbs: BoolExpr,
    ) -> FeltExpr {
        let mut low_limbs_value = low_limbs[0].clone();
        for (i, limb) in low_limbs.iter().enumerate().take(LIMBS_IN_M31).skip(1) {
            low_limbs_value =
                limb.clone() * const_expr!(1 << (i * FELT252_BITS_PER_WORD)) + low_limbs_value;
        }

        low_limbs_value
            - msb.as_felt()
            - const_expr!(1 << (LIMBS_IN_M31 * FELT252_BITS_PER_WORD)) * mid_limbs.as_felt()
    }

    // Receives sign bits and 3 low limbs, and returns a `felt252` that represents this relative immediate.
    pub fn small_to_felt252(
        low_limbs: [FeltExpr; LIMBS_IN_M31],
        msb: BoolExpr,
        mid_limbs: BoolExpr,
    ) -> Felt252Expr {
        let msb_limb = msb.as_felt() * const_expr!(0x100);
        let mid_limb_value = mid_limbs.as_felt() * const_expr!(0x1ff);
        let mut full_value_limbs = vec![];

        // Least significant three are stay as-is
        for limb in low_limbs.into_iter() {
            full_value_limbs.push(limb);
        }

        // Limbs 3-20 are all 0x0 or all 0x1ff
        for _ in LIMBS_IN_M31..21 {
            full_value_limbs.push(mid_limb_value.clone());
        }

        // Limb 21 is:
        // 0x0 if the MSB is not set (this also implies that limbs 3-20 are zero)
        // 0x88 if the MSB is set and limbs 3-20 are zero
        // 0x87 if the MSB is set and limbs 3-20 are 0x1ff
        full_value_limbs.push(const_expr!(0x88) * msb.as_felt() - mid_limbs.as_felt());

        // Limbs 22-26 are always zero
        for _ in 22..27 {
            full_value_limbs.push(const_expr!(0));
        }

        // Limb 27 is the most significant limb
        full_value_limbs.push(msb_limb);
        full_value_limbs.into()
    }

    pub fn read_rel_imm(&self, air_builder: &mut AirBuilder, address: FeltExpr) -> FeltExpr {
        air_builder
            .call(
                &ReadSmall {
                    memory: self.clone(),
                },
                address,
            )
            .0
    }

    pub fn felt252_to_addr(value: Felt252Expr) -> FeltExpr {
        let mut result = value.get_felt(0);

        for i in 1..(ADDRESS_BITS.div_ceil(FELT252_BITS_PER_WORD)) {
            result = result + value.get_felt(i) * const_expr!(1 << (FELT252_BITS_PER_WORD * i));
        }

        result
    }

    pub fn read_address(&self, air_builder: &mut AirBuilder, address: FeltExpr) -> FeltExpr {
        let (address_f252, _) = air_builder.call(
            &ReadPositive {
                memory: self.clone(),
                num_bits: ADDRESS_BITS,
            },
            address,
        );

        Self::felt252_to_addr(address_f252)
    }
}
