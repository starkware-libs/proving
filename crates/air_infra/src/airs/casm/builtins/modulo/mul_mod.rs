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
pub const MUL_MOD_MAX_LIMB: i32 = (1 << MUL_MOD_LIMB_SIZE) - 1;

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

        let [p_12bits, a_12bits, b_12bits, c_12bits] = [(p, "p"), (a, "a"), (b, "b"), (c, "c")]
            .map(|(x, desc)| {
                let res: [FeltExpr; MUL_MOD_NUM_LIMBS] = x
                    .into_iter()
                    .flat_map(|v| ab.call(&ModWordTo12BitArray {}, v))
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("Expected MUL_MOD_NUM_LIMBS limbs.");
                ab.let_(res, desc)
            });

        // TODO(ohadn): move BoundedFeltExpr to a neutral file and use it here.
        let mut limb_accumulator = const_expr!(0u32);
        let mut min_bound_acc = 0_i32;
        let mut max_bound_acc = 0_i32;
        for i in 0..(2 * MUL_MOD_NUM_LIMBS - 2) {
            if i < MUL_MOD_NUM_LIMBS {
                limb_accumulator = limb_accumulator - c_12bits[i].clone();
                min_bound_acc -= MUL_MOD_MAX_LIMB;
            }
            let convolution_start = max(i, MUL_MOD_NUM_LIMBS - 1) - (MUL_MOD_NUM_LIMBS - 1);
            let convolution_end = min(i, MUL_MOD_NUM_LIMBS - 1);
            for j in convolution_start..=convolution_end {
                limb_accumulator = limb_accumulator
                    + (a_12bits[j].clone() * b_12bits[i - j].clone()
                        - k_384.get_felt(j) * p_12bits[i - j].clone());
                max_bound_acc += MUL_MOD_MAX_LIMB * MUL_MOD_MAX_LIMB;
                min_bound_acc -= MUL_MOD_MAX_LIMB * MUL_MOD_MAX_LIMB;
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
        let limb1b = ab.deduce_air_var(mod_word_u16_arr[1].clone() >> const_u16_expr!(3), "limb1b");
        let limb1a = ab.let_(
            mod_word.get_felt(1) - (limb1b.as_felt() * const_expr!(1 << 3)),
            "limb1a",
        );

        result[0] = mod_word.get_felt(0) + const_expr!(1 << 9) * limb1a.clone();

        let limb2b = ab.deduce_air_var(mod_word_u16_arr[2].clone() >> const_u16_expr!(6), "limb2b");
        let limb2a = ab.let_(
            mod_word.get_felt(2) - (limb2b.as_felt() * const_expr!(1 << 6)),
            "limb2a",
        );
        result[1] = limb1b.as_felt() + const_expr!(1 << 6) * limb2a.clone();
        result[2] = limb2b.as_felt() + const_expr!(1 << 3) * mod_word.get_felt(3);

        range_check(
            ab,
            &[3, 6, 6, 3],
            &[limb1a, limb1b.as_felt(), limb2a, limb2b.as_felt()],
        );

        let limb5b = ab.deduce_air_var(mod_word_u16_arr[5].clone() >> const_u16_expr!(3), "limb5b");
        let limb5a = ab.let_(
            mod_word.get_felt(5) - (limb5b.as_felt() * const_expr!(1 << 3)),
            "limb5a",
        );
        result[3] = mod_word.get_felt(4) + const_expr!(1 << 9) * limb5a.clone();

        let limb6b = ab.deduce_air_var(mod_word_u16_arr[6].clone() >> const_u16_expr!(6), "limb6b");
        let limb6a = ab.let_(
            mod_word.get_felt(6) - (limb6b.as_felt() * const_expr!(1 << 6)),
            "limb6a",
        );
        result[4] = limb5b.as_felt() + const_expr!(1 << 6) * limb6a.clone();
        result[5] = limb6b.as_felt() + const_expr!(1 << 3) * mod_word.get_felt(7);

        range_check(
            ab,
            &[3, 6, 6, 3],
            &[limb5a, limb5b.as_felt(), limb6a, limb6b.as_felt()],
        );

        let limb9b = ab.deduce_air_var(mod_word_u16_arr[9].clone() >> const_u16_expr!(3), "limb9b");
        let limb9a = ab.let_(
            mod_word.get_felt(9) - (limb9b.as_felt() * const_expr!(1 << 3)),
            "limb9a",
        );

        result[6] = mod_word.get_felt(8) + const_expr!(1 << 9) * limb9a.clone();
        result[7] = limb9b.as_felt() + const_expr!(1 << 6) * mod_word.get_felt(10);

        // TODO(OhadN): Consider batching these into [3, 6, 6, 3] range checks.
        range_check(ab, &[3, 6], &[limb9a, limb9b.as_felt()]);

        result
    }
}
