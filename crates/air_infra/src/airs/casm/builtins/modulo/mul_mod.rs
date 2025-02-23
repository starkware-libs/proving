use core::array::from_fn;
use std::cmp::{max, min};

use compiled_casm_air::public_params::PublicParam;
use inst_def::InstDef;

use super::mod_utils::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::range_check::*;
use crate::airs::casm::const_tables::seq::*;
// Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::biguint_expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::variables::*;

pub const MUL_MOD_LIMB_SIZE: usize = 12;
pub const MUL_MOD_NUM_LIMBS: usize =
    (MOD_BUILTIN_N_WORDS * MOD_BUILTIN_WORD_BIT_LEN).div_ceil(MUL_MOD_LIMB_SIZE);
// We assume MOD_BUILTIN_WORD_BIT_LEN is a multiple of MUL_MOD_LIMB_SIZE.
pub const NUM_12BIT_LIMBS_PER_WORD: usize = MOD_BUILTIN_WORD_BIT_LEN.div_ceil(MUL_MOD_LIMB_SIZE);

#[derive(Debug, InstDef, Default)]
pub struct MulModBuiltin {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for MulModBuiltin {
    type ExtIn = ();
    type In = ();
    type Out = ();

    fn call(&self, ab: &mut AirBuilder, _: (), _: ()) -> Self::Out {
        let instance_num = ab.call_external_table(&Seq {});
        let segment_start = ab.get_public_param(PublicParam::MulModBuiltinSegmentStart);
        let [p, a, b, c] = ab.call(
            &ModUtils {
                memory: self.memory.clone(),
            },
            (
                CasmAddress::new(segment_start, "mul_mod_segment_start"),
                instance_num,
            ),
        );

        let shift = const_expr!(1 << MUL_MOD_LIMB_SIZE);
        let shift_inverse = const_expr!(1) / shift.clone();

        let a_384: BigUInt384Expr = a.to_vec().into();
        let b_384: BigUInt384Expr = b.to_vec().into();
        let c_384: BigUInt384Expr = c.to_vec().into();
        let p_384: BigUInt384Expr = p.to_vec().into();
        let p_768 = BigUInt768Expr::from(p_384);

        let mut k_768: BigUInt768Expr = a_384.clone().widening_mul(b_384.clone());

        k_768 = (k_768 - c_384.into()) / p_768;
        let mut k_384 = BigUInt384Expr::from(k_768);

        k_384 = ab.deduce_air_var(k_384, "ab_minus_c_div_p");
        for k_limb in k_384.as_felts() {
            range_check(ab, &[MUL_MOD_LIMB_SIZE as u16], &[k_limb]);
        }

        let [p_12bits, a_12bits, b_12bits, c_12bits] =
            [p, a, b, c].map(|x| mod_value_to_12bit_array(ab, x));

        // TODO(ohadn): move BoundedFeltExpr to a neutral file and use it here.
        let mut limb_accumulator = const_expr!(0u32);
        let mut min_bound_acc = 0_i32;
        let mut max_bound_acc = 0_i32;
        const MAX_WORD: i32 = (1 << MUL_MOD_LIMB_SIZE) - 1;
        for i in 0..(2 * MUL_MOD_NUM_LIMBS - 2) {
            if i < MUL_MOD_NUM_LIMBS {
                limb_accumulator = limb_accumulator - c_12bits[i].clone();
                min_bound_acc -= MAX_WORD;
            }
            let convolution_start = max(i, MUL_MOD_NUM_LIMBS - 1) - (MUL_MOD_NUM_LIMBS - 1);
            let convolution_end = min(i, MUL_MOD_NUM_LIMBS - 1);
            for j in convolution_start..=convolution_end {
                limb_accumulator = limb_accumulator
                    + (a_12bits[j].clone() * b_12bits[i - j].clone()
                        - k_384.get_felt(j) * p_12bits[i - j].clone());
                max_bound_acc += MAX_WORD * MAX_WORD;
                min_bound_acc -= MAX_WORD * MAX_WORD;
            }
            let carry = ab.assign(
                &mut (limb_accumulator.clone() * shift_inverse.clone()),
                "carry",
            );
            assert!(
                max_bound_acc < 1i32 << (2 * MUL_MOD_LIMB_SIZE + 5),
                "max_bound_acc exceeds 1 << (2 * MUL_MOD_LIMB_SIZE + 5)"
            );
            assert!(
                min_bound_acc >= -(1i32 << (2 * MUL_MOD_LIMB_SIZE + 5)),
                "abs(min_bound_acc) exceeds 1 << (2 * MUL_MOD_LIMB_SIZE + 5)"
            );

            // carry is possibly negative yet should be bound by 1u32 << (FELT252_BITS_PER_WORD + 5)
            // in absolute value
            range_check(
                ab,
                &[(MUL_MOD_LIMB_SIZE + 6) as u16],
                &[carry.clone() + const_expr!(1u32 << (MUL_MOD_LIMB_SIZE + 5))],
            );
            limb_accumulator = carry;
            // Maximal values of the carry that could satisfy the range check.
            max_bound_acc = (1i32 << (MUL_MOD_LIMB_SIZE + 5)) - 1;
            min_bound_acc = -(1i32 << (MUL_MOD_LIMB_SIZE + 5));
        }
        ab.constrain(
            a_12bits[MUL_MOD_NUM_LIMBS - 1].clone() * b_12bits[MUL_MOD_NUM_LIMBS - 1].clone()
                + limb_accumulator
                - k_384.get_felt(MUL_MOD_NUM_LIMBS - 1) * p_12bits[MUL_MOD_NUM_LIMBS - 1].clone(),
            "final limb constraint",
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }
}

pub fn mod_value_to_12bit_array(
    ab: &mut AirBuilder,
    mod_val: [Felt252Expr; MOD_BUILTIN_N_WORDS],
) -> [FeltExpr; MUL_MOD_NUM_LIMBS] {
    let mut result: [FeltExpr; MUL_MOD_NUM_LIMBS] = Default::default();
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
        let mut result: [FeltExpr; NUM_12BIT_LIMBS_PER_WORD] = Default::default();
        let mod_word_u16_arr: [UInt16Expr; N_SUBWORDS_IN_WORD] =
            from_fn(|i| UInt16Expr::from(mod_word.get_felt(i)));

        // TODO(ohadn): Consider using a loop here.
        let limb1b = ab.deduce_air_var(
            mod_word_u16_arr[1].clone() >> const_u16_expr!(3),
            "limb1b_u16",
        );
        let limb1a = mod_word.get_felt(1) - (limb1b.as_felt() * const_expr!(1 << 3));

        result[0] = ab.let_(
            mod_word.get_felt(0) + const_expr!(1 << 9) * limb1a.clone(),
            "res0",
        );

        let limb2b = ab.deduce_air_var(
            mod_word_u16_arr[2].clone() >> const_u16_expr!(6),
            "limb2b_u16",
        );
        let limb2a = mod_word.get_felt(2) - (limb2b.as_felt() * const_expr!(1 << 6));
        result[1] = ab.let_(
            limb1b.as_felt() + const_expr!(1 << 6) * limb2a.clone(),
            "res1",
        );
        result[2] = ab.let_(
            limb2b.as_felt() + const_expr!(1 << 3) * mod_word.get_felt(3),
            "res2",
        );

        range_check(
            ab,
            &[3, 6, 6, 3],
            &[
                limb1a.clone(),
                limb1b.as_felt(),
                limb2a.clone(),
                limb2b.as_felt(),
            ],
        );

        let limb5b = ab.deduce_air_var(
            mod_word_u16_arr[5].clone() >> const_u16_expr!(3),
            "limb5b_u16",
        );
        let limb5a = mod_word.get_felt(5) - (limb5b.as_felt() * const_expr!(1 << 3));
        result[3] = ab.let_(
            mod_word.get_felt(4) + const_expr!(1 << 9) * limb5a.clone(),
            "res3",
        );

        let limb6b = ab.deduce_air_var(
            mod_word_u16_arr[6].clone() >> const_u16_expr!(6),
            "limb6b_u16",
        );
        let limb6a = mod_word.get_felt(6) - (limb6b.as_felt() * const_expr!(1 << 6));
        result[4] = ab.let_(
            limb5b.as_felt() + const_expr!(1 << 6) * limb6a.clone(),
            "res4",
        );
        result[5] = ab.let_(
            limb6b.as_felt() + const_expr!(1 << 3) * mod_word.get_felt(7),
            "res5",
        );

        range_check(
            ab,
            &[3, 6, 6, 3],
            &[
                limb5a.clone(),
                limb5b.as_felt(),
                limb6a.clone(),
                limb6b.as_felt(),
            ],
        );

        let limb9b = ab.deduce_air_var(
            mod_word_u16_arr[9].clone() >> const_u16_expr!(3),
            "limb9b_u16",
        );
        let limb9a = mod_word.get_felt(9) - (limb9b.as_felt() * const_expr!(1 << 3));

        result[6] = ab.let_(
            mod_word.get_felt(8) + const_expr!(1 << 9) * limb9a.clone(),
            "res6",
        );
        result[7] = ab.let_(
            limb9b.as_felt() + const_expr!(1 << 6) * mod_word.get_felt(10),
            "res7",
        );

        // TODO(OhadN): Consider batching these into [3, 6, 6, 3] range checks.
        range_check(ab, &[3, 6], &[limb9a.clone(), limb9b.as_felt()]);

        result
    }
}
