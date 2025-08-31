use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_BITS_PER_WORD;

use super::memory::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::common::*;
// Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::felt252_id_memory::read_id::*;
use crate::core::felt252_id_memory::read_positive::*;

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

/// Receives a felt252, and conditionally constrains its sign bits as a relative-immediate
/// (the "case" bits: msb and mid_limbs_set).
/// Returns the deduced sign bits.
#[derive(Clone, Debug, Serialize)]
pub struct CondDecodeSmallSign {}

impl AirFn for CondDecodeSmallSign {
    type ExtIn = ();
    type In = (Felt252Expr, FeltExpr);
    type Out = [FeltExpr; 2];

    fn call(&self, air_builder: &mut AirBuilder, _: (), (value, condition): Self::In) -> Self::Out {
        let msb = air_builder.deduce_air_var(value.get_felt(27).eq(const_expr!(0x100)), "msb");
        let mid_limbs_set =
            air_builder.deduce_air_var(value.get_felt(20).eq(const_expr!(0x1ff)), "mid_limbs_set");

        // Require case bits to be bits
        air_builder.constrain(
            msb.as_felt() * (msb.as_felt() - const_expr!(1)),
            "msb is a bit",
        );
        air_builder.constrain(
            mid_limbs_set.as_felt() * (mid_limbs_set.as_felt() - const_expr!(1)),
            "mid_limbs_set is a bit",
        );

        // Forbid the case msb = 0, mid_limbs_set = 1
        air_builder.constrain(
            condition * mid_limbs_set.as_felt() * (msb.as_felt() - const_expr!(1)),
            "Cannot have msb equals 0 and mid_limbs_set equals 1",
        );

        [msb.as_felt(), mid_limbs_set.as_felt()]
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
    let limbs = low_limbs
        .into_iter()
        .chain([remainder_bits])
        .collect::<Vec<_>>();
    let low_limbs_value = felt252_to_m31(limbs.into(), SMALL_BITS);
    low_limbs_value - msb - const_expr!(1 << SMALL_BITS) * mid_limbs_set
}

// Receives sign bits, the low limbs and the remainder bits in the next limb.
// Returns a `felt252` that represents this relative immediate.
pub fn small_to_felt252(
    low_limbs: [FeltExpr; LIMBS_IN_SMALL],
    remainder_bits: FeltExpr,
    msb: FeltExpr,
    mid_limbs_set: FeltExpr,
) -> Felt252Expr {
    let msb_limb = msb.clone() * const_expr!(0x100);
    let mid_limb_value = mid_limbs_set.clone() * const_expr!(0x1ff);

    // Represent the limbs of the full value as linear combinations of the input felts.
    let mut full_value_limbs = vec![];

    // Least significant three stay as-is
    full_value_limbs.append(low_limbs.to_vec().as_mut());

    // Bits 28 and 29 stay as-is and bits 30-36 are 1 if mid_limbs_set and 0 otherwise.
    full_value_limbs.push(remainder_bits + (mid_limbs_set.clone() * const_expr!(0x1FC)));

    // Limbs 4-20 are all 0x0 or all 0x1ff
    for _ in (LIMBS_IN_SMALL + 1)..21 {
        full_value_limbs.push(mid_limb_value.clone());
    }

    // Limb 21 is:
    // 0x0 if the MSB is not set (this also implies that limbs 4-20 are zero)
    // 0x88 if the MSB is set and limbs 4-20 are zero
    // 0x87 if the MSB is set and limbs 4-20 are 0x1ff
    full_value_limbs.push(const_expr!(0x88) * msb - mid_limbs_set);

    // Limbs 22-26 are always zero
    for _ in 22..27 {
        full_value_limbs.push(const_expr!(0));
    }

    // Limb 27 is the most significant limb
    full_value_limbs.push(msb_limb);

    full_value_limbs.into()
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
        let id = air_builder.call(
            &ReadId {
                memory: self.memory.clone(),
            },
            address.clone(),
        );
        let mut value = air_builder.mem_read_unverified(&self.memory.id_to_big, &id);

        // Compute and deduce "case" bits: msb and mid_limbs_set
        let [msb, mid_limbs_set] =
            air_builder.call(&CondDecodeSmallSign {}, (value.clone(), const_expr!(1)));

        // Least significant three are deduced as-is
        let mut low_value_limbs = vec![];
        for i in 0..LIMBS_IN_SMALL {
            low_value_limbs.push(
                air_builder.deduce(
                    value.get_felt_mut(i),
                    &address
                        .extra_info
                        .clone()
                        .map(|s| format!("{}_limb_{}", s, i))
                        .unwrap_or(format!("value_limb_{}", i)),
                ),
            );
        }

        let remainder_bits = air_builder.deduce_air_var(
            UInt16Expr::from(value.get_felt(3)) & const_u16_expr!(0b11),
            "remainder_bits",
        );

        air_builder.call(
            &CondRangeCheck2 {},
            [remainder_bits.as_felt(), const_expr!(1)],
        );

        let low_limbs_arr: [FeltExpr; LIMBS_IN_SMALL] = low_value_limbs
            .try_into()
            .expect("Incorrect size for the low value");

        // Verify that the value in memory is the one we expect
        air_builder.mem_verify(
            &self.memory.id_to_big,
            &id,
            small_to_felt252(
                low_limbs_arr.clone(),
                remainder_bits.as_felt(),
                msb.clone(),
                mid_limbs_set.clone(),
            ),
        );

        (
            small_to_rel_imm(low_limbs_arr, remainder_bits.as_felt(), msb, mid_limbs_set),
            id,
        )
    }
}
