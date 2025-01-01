use core::array::from_fn;

use inst_def::InstDef;

use super::mod_utils::*;
use crate::airs::casm::const_tables::range_check::*;
// Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;

pub const MUL_MOD_LIMB_SIZE: usize = 12;
pub const MUL_MOD_NUM_LIMBS: usize =
    (MOD_BUILTIN_N_WORDS * MOD_BUILTIN_WORD_BIT_LEN).div_ceil(MUL_MOD_LIMB_SIZE);
// We assume MOD_BUILTIN_WORD_BIT_LEN is a multiple of MUL_MOD_LIMB_SIZE.
pub const NUM_12BIT_LIMBS_PER_WORD: usize = MOD_BUILTIN_WORD_BIT_LEN.div_ceil(MUL_MOD_LIMB_SIZE);

pub fn mod_value_to_12bit_array(
    ab: &mut AirBuilder,
    mod_val: [Felt252Expr; MOD_BUILTIN_N_WORDS],
) -> [FeltExpr; MUL_MOD_NUM_LIMBS] {
    let mut result: [FeltExpr; MUL_MOD_NUM_LIMBS] = from_fn(|_| FeltExpr::default());
    for i in 0..MOD_BUILTIN_N_WORDS {
        result[NUM_12BIT_LIMBS_PER_WORD * i..NUM_12BIT_LIMBS_PER_WORD * (i + 1)]
            .clone_from_slice(&ab.call(&ModWordTo12BitArray {}, mod_val[i].clone()));
    }
    result
}

#[derive(Debug, InstDef, Default)]
pub struct ModWordTo12BitArray {}

impl AirFn for ModWordTo12BitArray {
    type ExtIn = ();
    type In = Felt252Expr;
    type Out = [FeltExpr; NUM_12BIT_LIMBS_PER_WORD];

    fn call(&self, ab: &mut AirBuilder, _: (), mod_word: Self::In) -> Self::Out {
        let mut result: [FeltExpr; NUM_12BIT_LIMBS_PER_WORD] = from_fn(|_| FeltExpr::default());
        let mod_word_u16_arr: [UInt16Expr; N_SUBWORDS_IN_WORD] =
            from_fn(|i| UInt16Expr::from(mod_word.get_felt(i)));

        let mut limb1b_u16 = ab.let_for_deduction(
            mod_word_u16_arr[1].clone() >> const_u16_expr!(3),
            "limb1b_u16",
        );
        let limb1b = ab.deduce(limb1b_u16.as_felt_mut(), "limb1b");
        let limb1a = mod_word.get_felt(1) - (limb1b.clone() * const_expr!(1 << 3));

        result[0] = mod_word.get_felt(0) + const_expr!(1 << 9) * limb1a.clone();

        let mut limb2b_u16 = ab.let_for_deduction(
            mod_word_u16_arr[2].clone() >> const_u16_expr!(6),
            "limb2b_u16",
        );
        let limb2b = ab.deduce(limb2b_u16.as_felt_mut(), "limb2b");
        let limb2a = mod_word.get_felt(2) - (limb2b.clone() * const_expr!(1 << 6));
        result[1] = limb1b.clone() + const_expr!(1 << 6) * limb2a.clone();
        result[2] = limb2b.clone() + const_expr!(1 << 3) * mod_word.get_felt(3);

        ab.lookup_call(
            &RangeCheck::<RangeCheck3_6_6_3>::default(),
            [
                limb1a.clone(),
                limb1b.clone(),
                limb2a.clone(),
                limb2b.clone(),
            ],
            (),
        );

        let mut limb5b_u16 = ab.let_for_deduction(
            mod_word_u16_arr[5].clone() >> const_u16_expr!(3),
            "limb5b_u16",
        );
        let limb5b = ab.deduce(limb5b_u16.as_felt_mut(), "limb5b");
        let limb5a = mod_word.get_felt(5) - (limb5b.clone() * const_expr!(1 << 3));
        result[3] = mod_word.get_felt(4) + const_expr!(1 << 9) * limb5a.clone();

        let mut limb6b_u16 = ab.let_for_deduction(
            mod_word_u16_arr[6].clone() >> const_u16_expr!(6),
            "limb6b_u16",
        );
        let limb6b = ab.deduce(limb6b_u16.as_felt_mut(), "limb6b");
        let limb6a = mod_word.get_felt(6) - (limb6b.clone() * const_expr!(1 << 6));
        result[4] = limb5b.clone() + const_expr!(1 << 6) * limb6a.clone();
        result[5] = limb6b.clone() + const_expr!(1 << 3) * mod_word.get_felt(7);

        ab.lookup_call(
            &RangeCheck::<RangeCheck3_6_6_3>::default(),
            [
                limb5a.clone(),
                limb5b.clone(),
                limb6a.clone(),
                limb6b.clone(),
            ],
            (),
        );

        let mut limb9b_u16 = ab.let_for_deduction(
            mod_word_u16_arr[9].clone() >> const_u16_expr!(3),
            "limb9b_u16",
        );
        let limb9b = ab.deduce(limb9b_u16.as_felt_mut(), "limb9b");
        let limb9a = mod_word.get_felt(9) - (limb9b.clone() * const_expr!(1 << 3));

        result[6] = mod_word.get_felt(8) + const_expr!(1 << 9) * limb9a.clone();
        result[7] = limb9b.clone() + const_expr!(1 << 6) * mod_word.get_felt(10);

        // TODO(OhadN): Consider batching these into [3, 6, 6, 3] range checks.
        ab.lookup_call(
            &RangeCheck::<RangeCheck3_6>::default(),
            [limb9a.clone(), limb9b.clone()],
            (),
        );

        result
    }
}
