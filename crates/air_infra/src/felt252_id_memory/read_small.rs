use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_BITS_PER_WORD;

use super::memory::*;
use crate::casm_state::*;
// Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::felt252_id_memory::read_id::*;
use crate::felt252_id_memory::read_positive::*;
use crate::utils::*;

// The number of bits in a "small" value.
pub const SMALL_BITS: usize = ADDRESS_BITS;
// The number of whole limbs that fit in a "small" value. When reading a "small" value into an M31
// we'll deduce that many limbs.
pub const LIMBS_IN_SMALL: usize = SMALL_BITS / FELT252_BITS_PER_WORD;

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

/// Receives a felt252 and constrains its sign bits as a relative-immediate
/// (the "case" bits: msb and mid_limbs_set).
/// Returns the dedeuced sign bits and the four values needed to construct the relative immediate
/// besides the low-limbs value: the 7 high bits of limb 3, limbs 4–20, limb 21, and limb 27.
/// If the given felt252 is not a small value, the mid_limbs_set will be set to zero, (i.e a garbage
/// small value will be calculated out of this case bits).
#[derive(Clone, Debug, Serialize)]
pub struct DecodeSmallSign {}

impl AirFn for DecodeSmallSign {
    type ExtIn = ();
    type In = Felt252Expr;
    type Out = [FeltExpr; 6];

    fn call(&self, air_builder: &mut AirBuilder, _: (), value: Self::In) -> Self::Out {
        let msb = air_builder.deduce_air_var(value.get_felt(27).eq(const_expr!(0x100)), "msb");

        // We apply a bitwise AND with msb_bool to ensure that when msb_bool is 0, mid_limbs_set is
        // also 0.
        let mid_limbs_set = air_builder.deduce_air_var(
            value.get_felt(20).eq(const_expr!(0x1ff)) & msb.clone(),
            "mid_limbs_set",
        );

        // Require case bits to be bits
        air_builder.constrain(msb.as_felt() * (msb.as_felt() - const_expr!(1)), "msb is a bit");
        air_builder.constrain(
            mid_limbs_set.as_felt() * (mid_limbs_set.as_felt() - const_expr!(1)),
            "mid_limbs_set is a bit",
        );

        // Forbid the case msb = 0, mid_limbs_set = 1
        air_builder.constrain(
            mid_limbs_set.as_felt() * (msb.as_felt() - const_expr!(1)),
            "Cannot have msb equals 0 and mid_limbs_set equals 1",
        );

        // Bits 30-36 (7 high bits of limb 3) are 1 if mid_limbs_set and 0 otherwise
        let limb3_7_high_bits = mid_limbs_set.as_felt() * const_expr!(0x1FC);

        // Limbs 4-20 are 0x0 or all 0x1ff
        let limbs4_to_20 = mid_limbs_set.as_felt() * const_expr!(0x1ff);

        // Limb 21 is:
        // 0x0 if the MSB is not set (this also implies that limbs 4-20 are zero)
        // 0x88 if the MSB is set and limbs 4-20 are zero
        // 0x87 if the MSB is set and limbs 4-20 are 0x1ff
        let limb21 = msb.as_felt() * const_expr!(0x88) - mid_limbs_set.as_felt();

        // Limb 27 is either 0x0 or 0x100
        let limb27 = msb.as_felt() * const_expr!(0x100);

        [msb.as_felt(), mid_limbs_set.as_felt(), limb3_7_high_bits, limbs4_to_20, limb21, limb27]
    }

    fn output_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("msb".to_string()),
            Some("mid_limbs_set".to_string()),
            Some("limb3_7_high_bits".to_string()),
            Some("limbs4_to_20".to_string()),
            Some("limb21".to_string()),
            Some("limb27".to_string()),
        ])
    }
}

// Receives sign bits, the low limbs, and the remainder bits in the next limb.
// Returns a FeltExpr that represents this relative immediate.
pub fn small_to_rel_imm(
    low_limbs: [FeltExpr; LIMBS_IN_SMALL],
    remainder_bits: FeltExpr,
    msb: FeltExpr,
    mid_limbs_set: FeltExpr,
) -> FeltExpr {
    let limbs = low_limbs.into_iter().chain([remainder_bits]).collect::<Vec<_>>();
    let low_limbs_value = felt252_to_m31(limbs.into(), SMALL_BITS);
    low_limbs_value - msb - const_expr!(1 << SMALL_BITS) * mid_limbs_set
}

#[derive(Debug, Serialize, Default)]
pub struct ReadSmall {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

/// Read a Felt252 that has a small magnitude into a Felt. The allowed range
/// for the Felt252 is [-2**29 - 1, 2**29 - 1] (for 9-bit limbs).
/// Returns also the ID of the value in the memory.
impl AirFn for ReadSmall {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = (FeltExpr, CasmId);

    fn call(&self, air_builder: &mut AirBuilder, _: (), address: Self::In) -> Self::Out {
        let id = air_builder.call(&ReadId { memory: self.memory.clone() }, address.clone());
        let mut value = air_builder.mem_read_unverified(&self.memory.id_to_big, &id);

        // Compute the four values needed to construct the relative immediate other then the
        // low-limbs value.
        let [msb, mid_limbs_set, limb3_7_high_bits, limbs4_to_20, limb21, limb27] =
            air_builder.call(&DecodeSmallSign {}, value.clone());

        // Least significant three are deduced as-is
        let mut expected_value = vec![];
        for i in 0..LIMBS_IN_SMALL {
            // Push limbs 0-2
            expected_value.push(
                air_builder.deduce(
                    value.get_felt_mut(i),
                    &address
                        .extra_info
                        .clone()
                        .map(|s| format!("{s}_limb_{i}"))
                        .unwrap_or(format!("value_limb_{i}")),
                ),
            );
        }

        let remainder_bits = air_builder.deduce_air_var(
            UInt16Expr::from(value.get_felt(LIMBS_IN_SMALL)) & const_u16_expr!(0b11),
            "remainder_bits",
        );
        air_builder.call(&CondRangeCheck2 {}, [remainder_bits.as_felt(), const_expr!(1)]);

        // Push limb 3
        expected_value.push(remainder_bits.as_felt() + limb3_7_high_bits);

        // Push limbs 4-20
        for _ in (LIMBS_IN_SMALL + 1)..21 {
            expected_value.push(limbs4_to_20.clone());
        }

        // Push limb 21
        expected_value.push(limb21);

        // Limbs 22-26 are always zero
        for _ in 22..27 {
            expected_value.push(const_expr!(0));
        }

        // Push limb 27
        expected_value.push(limb27.clone());

        // Verify that the value in memory is the one we expect
        air_builder.mem_verify(&self.memory.id_to_big, &id, expected_value.clone().into());

        (
            small_to_rel_imm(
                expected_value[..LIMBS_IN_SMALL]
                    .to_vec()
                    .try_into()
                    .expect("Incorrect size for the low value"),
                remainder_bits.as_felt(),
                msb,
                mid_limbs_set,
            ),
            id,
        )
    }
}
