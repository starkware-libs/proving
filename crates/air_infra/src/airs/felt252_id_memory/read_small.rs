use inst_def::InstDef;

use super::memory::*;

use crate::airs::casm::common::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::prover_types::*;

// Macros
use crate::const_expr;

// The number of limbs that fit in an M31. When reading a "small" value into an M31
// we'll deduce that many limbs.
pub const LIMBS_IN_M31: usize = 3;

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

/// Receives a felt252, deduces, and conditionally constrains its sign bits as a relative-immediate
/// (the "case" bits: msb and mid_limbs_set).
/// Returns the deduced sign bits.
#[derive(Clone, Debug, InstDef)]
pub struct CondDecodeSmallSign {}

impl AirFn for CondDecodeSmallSign {
    type In = (Felt252Expr, FeltExpr);
    type Out = [FeltExpr; 2];

    fn call(&self, air_builder: &mut AirBuilder, (value, condition): Self::In) -> Self::Out {
        let mut msb_bool = air_builder.let_for_deduction(value.get_felt(27).eq(const_expr!(0x100)));
        let msb = air_builder.deduce(msb_bool.as_felt_mut(), "msb");
        let mut mid_limbs_set_bool =
            air_builder.let_for_deduction(value.get_felt(20).eq(const_expr!(0x1ff)));
        let mid_limbs_set = air_builder.deduce(mid_limbs_set_bool.as_felt_mut(), "mid_limbs_set");

        // Require case bits to be bits
        air_builder.constrain(msb.clone() * (msb.clone() - const_expr!(1)));
        air_builder.constrain(mid_limbs_set.clone() * (mid_limbs_set.clone() - const_expr!(1)));

        // Forbid the case msb = 0, mid_limbs_set = 1
        air_builder.constrain(condition * mid_limbs_set.clone() * (msb.clone() - const_expr!(1)));

        [msb, mid_limbs_set]
    }
}

// Receives a Felt252 and its sign bits, and returns it as a relative immediate felt.
pub fn small_to_rel_imm(
    low_limbs: [FeltExpr; LIMBS_IN_M31],
    msb: FeltExpr,
    mid_limbs_set: FeltExpr,
) -> FeltExpr {
    let mut low_limbs_value = low_limbs[0].clone();

    for (i, limb) in low_limbs.into_iter().enumerate().skip(1) {
        low_limbs_value = limb * const_expr!(1 << (i * FELT252_BITS_PER_WORD)) + low_limbs_value;
    }

    low_limbs_value - msb - const_expr!(1 << (LIMBS_IN_M31 * FELT252_BITS_PER_WORD)) * mid_limbs_set
}

// Receives sign bits and 3 low limbs, and returns a `felt252` that represents this relative immediate.
pub fn small_to_felt252(
    low_limbs: [FeltExpr; LIMBS_IN_M31],
    msb: FeltExpr,
    mid_limbs_set: FeltExpr,
) -> Felt252Expr {
    let msb_limb = msb.clone() * const_expr!(0x100);
    let mid_limb_value = mid_limbs_set.clone() * const_expr!(0x1ff);

    // Represent the limbs of the full value as linear combinations of the input felts.
    let mut full_value_limbs = vec![];

    // Least significant three stay as-is
    full_value_limbs.append(low_limbs.to_vec().as_mut());

    // Limbs 3-20 are all 0x0 or all 0x1ff
    for _ in LIMBS_IN_M31..21 {
        full_value_limbs.push(mid_limb_value.clone());
    }

    // Limb 21 is:
    // 0x0 if the MSB is not set (this also implies that limbs 3-20 are zero)
    // 0x88 if the MSB is set and limbs 3-20 are zero
    // 0x87 if the MSB is set and limbs 3-20 are 0x1ff
    full_value_limbs.push(const_expr!(0x88) * msb - mid_limbs_set);

    // Limbs 22-26 are always zero
    for _ in 22..27 {
        full_value_limbs.push(const_expr!(0));
    }

    // Limb 27 is the most significant limb
    full_value_limbs.push(msb_limb);

    full_value_limbs.into()
}

#[derive(Debug, InstDef, Default)]
pub struct ReadSmall {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

/// Read a Felt252 that has a small magnitude into a Felt. The allowed range
/// for the Felt252 is [-2**27, 2**27 - 1] (for 9-bit limbs).
/// Returns also the ID of the value in the memory.
impl AirFn for ReadSmall {
    type In = CasmAddress;
    type Out = (FeltExpr, FeltExpr);

    fn call(&self, air_builder: &mut AirBuilder, address: Self::In) -> Self::Out {
        let mut id = air_builder.mem_read_unverified(&self.memory.address_to_id, &address);
        air_builder.deduce(&mut id, "id");
        air_builder.mem_verify(&self.memory.address_to_id, &address, id.clone());
        let mut value = air_builder.mem_read_unverified(&self.memory.id_to_value, &id);

        // Compute and deduce "case" bits: msb and mid_limbs_set
        let [msb, mid_limbs_set] =
            air_builder.call(&CondDecodeSmallSign {}, (value.clone(), const_expr!(1)));

        // Least significant three are deduced as-is
        let mut low_value_limbs = vec![];
        for i in 0..LIMBS_IN_M31 {
            low_value_limbs.push(air_builder.deduce(value.get_felt_mut(i), &format!("limb_{}", i)));
        }
        let low_limbs_arr: [FeltExpr; LIMBS_IN_M31] = low_value_limbs
            .try_into()
            .expect("Incorrect size for the low value");

        // Verify that the value in memory is the one we expect
        air_builder.mem_verify(
            &self.memory.id_to_value,
            &id,
            small_to_felt252(low_limbs_arr.clone(), msb.clone(), mid_limbs_set.clone()),
        );

        (small_to_rel_imm(low_limbs_arr, msb, mid_limbs_set), id)
    }
}
