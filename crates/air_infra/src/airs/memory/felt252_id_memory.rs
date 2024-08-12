#[cfg(test)]
use std::collections::BTreeMap;

use indexmap::IndexMap;

use crate::airs::casm::common::*;
use crate::airs::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

#[cfg(test)]
use crate::core::Felt;

#[cfg(test)]
use crate::expr;

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

// The number of limbs that fit in an M31. When reading a "small" value into an M31
// we'll deduce that many limbs.
const LIMBS_IN_M31: usize = 3;

#[derive(Debug)]
pub struct ReadSmall {
    pub memory: Felt252IdMemory,
}

// 9-bit limbs
//  limb ->   27  26  25  24  23  22  21  20  19 ...   5   4   3   2   1   0
// value
// 2 (+P)  0x100 000 000 000 000 000 088 000 000 ... 000 000 000 000 000 003
// 1 (+P)  0x100 000 000 000 000 000 088 000 000 ... 000 000 000 000 000 002
// 0 (+P)  0x100 000 000 000 000 000 088 000 000 ... 000 000 000 000 000 001
// 2       0x000 000 000 000 000 000 000 000 000 ... 000 000 000 000 000 002
// 1       0x000 000 000 000 000 000 000 000 000 ... 000 000 000 000 000 001
// 0       0x000 000 000 000 000 000 000 000 000 ... 000 000 000 000 000 000
// -1      0x100 000 000 000 000 000 088 000 000 ... 000 000 000 000 000 000
// -2      0x100 000 000 000 000 000 087 1ff 1ff ... 1ff 1ff 1ff 1ff 1ff 1ff
// -3      0x100 000 000 000 000 000 087 1ff 1ff ... 1ff 1ff 1ff 1ff 1ff 1fe

/// Read a Felt252 that has a small magnitude into a Felt. The allowed range
/// for the Felt252 is [-2**24, 2**24 - 1] (for 12-bit limbs).
impl AirFn for ReadSmall {
    type In = CasmAddress;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, address: Self::In) -> Self::Out {
        let mut id = air_builder.get_from_memory(&self.memory.address_to_id, &address);
        air_builder.deduce(&mut id);
        air_builder.set_in_memory(&self.memory.address_to_id, address, id.clone());
        let mut value = air_builder.get_from_memory(&self.memory.id_to_value, &id);

        // Compute and deduce "case" bits: msb and mid_limbs_set
        let mut msb_bool = air_builder.let_for_deduction(value.get_felt(27).eq(const_expr!(0x100)));
        let msb = air_builder.deduce(msb_bool.as_felt_mut());
        let mut mid_limbs_set_bool =
            air_builder.let_for_deduction(value.get_felt(20).eq(const_expr!(0x1ff)));
        let mid_limbs_set = air_builder.deduce(mid_limbs_set_bool.as_felt_mut());

        // Require case bits to be bits
        air_builder.constrain(msb.clone() * (msb.clone() - const_expr!(1)));
        air_builder.constrain(mid_limbs_set.clone() * (mid_limbs_set.clone() - const_expr!(1)));

        // Forbid the case msb = 0, mid_limbs_set = 1
        air_builder.constrain(mid_limbs_set.clone() * (msb.clone() - const_expr!(1)));

        let msb_limb = msb.clone() * const_expr!(0x100);
        let mid_limb_value = mid_limbs_set.clone() * const_expr!(0x1ff);

        // Represent the limbs of the full in-memory value as linear combinations of the felts
        // we deduced to trace.
        let mut full_value_limbs = vec![];

        // Least significant three are deduced as-is
        for i in 0..LIMBS_IN_M31 {
            full_value_limbs.push(air_builder.deduce(value.get_felt_mut(i)));
        }

        // Limbs 3-20 are all 0x0 or all 0x1ff
        for _ in LIMBS_IN_M31..21 {
            full_value_limbs.push(mid_limb_value.clone());
        }

        // Limb 21 is:
        // 0x0 if the MSB is not set (this also implies that limbs 3-20 are zero)
        // 0x88 if the MSB is set and limbs 3-20 are zero
        // 0x87 if the MSB is set and limbs 3-20 are 0x1ff
        full_value_limbs.push(const_expr!(0x88) * msb.clone() - mid_limbs_set.clone());

        // Limbs 22-26 are always zero
        for _ in 22..27 {
            full_value_limbs.push(const_expr!(0));
        }

        // Limb 27 is the most significant limb
        full_value_limbs.push(msb_limb);

        // Verify that the value in memory is the one we expect
        air_builder.set_in_memory(&self.memory.id_to_value, id, full_value_limbs.into());

        let mut low_limbs_value = value.get_felt(0).clone();
        for i in 1..LIMBS_IN_M31 {
            low_limbs_value =
                value.get_felt(i) * const_expr!(1 << (i * FELT252_BITS_PER_WORD)) + low_limbs_value;
        }
        low_limbs_value
            - msb.clone()
            - const_expr!(1 << (LIMBS_IN_M31 * FELT252_BITS_PER_WORD)) * mid_limbs_set.clone()
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
                    bits: [bits_in_ms_limb as u16],
                },
                [value
                    .as_felts()
                    .into_iter()
                    .nth(num_nonzero_limbs - 1)
                    .expect("The value should have enough limbs")],
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
