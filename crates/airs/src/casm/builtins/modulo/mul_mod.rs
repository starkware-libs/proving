use core::array::from_fn;

use air_common::TraceType;
use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::biguint_expr::{BigUInt384Expr, BigUInt768Expr};
use air_infra::core::expressions::bounded_felt::BoundedFeltExpr;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::core::public_params::PublicParam;
use air_infra::core::variables::AirVar;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::range_check::range_check;
use air_infra::seq::Seq;
use air_infra::{const_expr, const_u16_expr};
use serde::Serialize;

use super::mod_utils::*;
use crate::convolution_utils::karatsuba::*;

pub const MUL_MOD_LIMB_SIZE: usize = 12;
pub const MUL_MOD_NUM_LIMBS: usize = {
    assert!(
        MOD_BUILTIN_WORD_BIT_LEN.is_multiple_of(MUL_MOD_LIMB_SIZE),
        "Mul mod word bit length must be divisible by mul mod limb size"
    );
    MOD_BUILTIN_N_WORDS * MOD_BUILTIN_WORD_BIT_LEN / MUL_MOD_LIMB_SIZE
};
pub const NUM_12BIT_LIMBS_PER_WORD: usize = MOD_BUILTIN_WORD_BIT_LEN.div_ceil(MUL_MOD_LIMB_SIZE);
pub const MUL_MOD_MAX_LIMB: i32 = (1 << MUL_MOD_LIMB_SIZE) - 1;
pub const MUL_MOD_KARATSUBA_N: usize = {
    assert!(MUL_MOD_NUM_LIMBS.is_multiple_of(4), "Mul mod number of limbs must be divisible by 4");
    MUL_MOD_NUM_LIMBS / 4
};

#[derive(Debug, Serialize, Default)]
pub struct MulModBuiltin {
    #[serde(skip)]
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
            &ModUtils { memory: self.memory.clone() },
            (CasmAddress::new(segment_start, "mul_mod_segment_start"), instance_num),
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

        let [p_12bits, a_12bits, b_12bits, c_12bits] = [p, a, b, c].map(|x| {
            let res: [FeltExpr; MUL_MOD_NUM_LIMBS] = x
                .chunks(2)
                .flat_map(|chunk| {
                    ab.call(&ModWordsTo12BitArray {}, [chunk[0].clone(), chunk[1].clone()])
                })
                .collect::<Vec<_>>()
                .try_into()
                .expect("Expected MUL_MOD_NUM_LIMBS limbs.");
            res
        });

        let mut limb_accumulator = BoundedFeltExpr::default();

        // Compute the convolutions a * b and k * p using Karatsuba.
        let karatsuba = DoubleKaratsuba::<{ MUL_MOD_NUM_LIMBS / 4 }>::new(
            MUL_MOD_MAX_LIMB * MUL_MOD_MAX_LIMB,
            0,
        );

        let a_mul_b_array = ab.call(&karatsuba, [a_12bits, b_12bits]);
        let k_mul_p_array = ab.call(
            &karatsuba,
            [
                k_384
                    .as_felts()
                    .try_into()
                    .unwrap_or_else(|_| panic!("k_384 should be of length {MUL_MOD_NUM_LIMBS}")),
                p_12bits,
            ],
        );

        for i in 0..(2 * MUL_MOD_NUM_LIMBS - 2) {
            if i < MUL_MOD_NUM_LIMBS {
                limb_accumulator -= (c_12bits[i].clone(), MUL_MOD_MAX_LIMB, 0).into();
            }
            limb_accumulator += a_mul_b_array[i].clone() - k_mul_p_array[i].clone();

            let mut carry = BoundedFeltExpr::new(
                ab.assign(
                    &mut (limb_accumulator.var.clone() * shift_inverse.clone()),
                    &format!("carry_{i}"),
                ),
                limb_accumulator.max_bound() >> MUL_MOD_LIMB_SIZE,
                limb_accumulator.min_bound() >> MUL_MOD_LIMB_SIZE,
            );

            // carry is possibly negative yet should be bound by 1u32 << (FELT252_BITS_PER_WORD + 5)
            // in absolute value
            assert!(carry.max_bound() < (1i32 << (MUL_MOD_LIMB_SIZE + 5)));
            assert!(carry.min_bound() >= -(1i32 << (MUL_MOD_LIMB_SIZE + 5)));
            range_check(
                ab,
                &[(MUL_MOD_LIMB_SIZE + 6) as u16],
                &[carry.var.clone() + const_expr!(1u32 << (MUL_MOD_LIMB_SIZE + 5))],
            );

            // Bounds on the carry based on the range-check constraint.
            carry.set_max_bound((1i32 << (MUL_MOD_LIMB_SIZE + 5)) - 1);
            carry.set_min_bound(1i32 << (MUL_MOD_LIMB_SIZE + 5));

            limb_accumulator = carry;
        }
        ab.constrain(
            a_mul_b_array[2 * MUL_MOD_NUM_LIMBS - 2].var.clone() + limb_accumulator.var
                - k_mul_p_array[2 * MUL_MOD_NUM_LIMBS - 2].var.clone(),
            "final limb constraint",
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }
}

#[derive(Debug, Serialize, Default)]
pub struct ModWordsTo12BitArray {}

impl AirFn for ModWordsTo12BitArray {
    type ExtIn = ();
    type In = [Felt252Expr; 2];
    type Out = [FeltExpr; 2 * NUM_12BIT_LIMBS_PER_WORD];

    fn call(&self, ab: &mut AirBuilder, _: (), mod_words: Self::In) -> Self::Out {
        let mut result: [FeltExpr; 2 * NUM_12BIT_LIMBS_PER_WORD] = Default::default();

        let mut to_range_check = vec![];

        for (word_index, word) in mod_words.iter().enumerate() {
            let word_offset = word_index * NUM_12BIT_LIMBS_PER_WORD;
            let mod_words_u16_arr: [UInt16Expr; N_SUBWORDS_IN_WORD] =
                from_fn(|i| word.get_felt(i).into());

            for k in 0..3 {
                let read_offset = 4 * k;
                let write_offset = 3 * k;

                let limb1b = ab.deduce_air_var(
                    mod_words_u16_arr[read_offset + 1].clone() >> const_u16_expr!(3),
                    &format!("limb{}b_{}", read_offset + 1, word_index),
                );
                let limb1a = ab.let_(
                    word.get_felt(read_offset + 1) - (limb1b.as_felt() * const_expr!(1 << 3)),
                    &format!("limb{}a_{}", read_offset + 1, word_index),
                );

                result[word_offset + write_offset] =
                    word.get_felt(read_offset) + const_expr!(1 << 9) * limb1a.clone();

                if k < 2 {
                    let limb2b = ab.deduce_air_var(
                        mod_words_u16_arr[read_offset + 2].clone() >> const_u16_expr!(6),
                        &format!("limb{}b_{}", read_offset + 2, word_index),
                    );
                    let limb2a = ab.let_(
                        word.get_felt(read_offset + 2) - (limb2b.as_felt() * const_expr!(1 << 6)),
                        &format!("limb{}a_{}", read_offset + 2, word_index),
                    );
                    result[word_offset + write_offset + 1] =
                        limb1b.as_felt() + const_expr!(1 << 6) * limb2a.clone();
                    result[word_offset + write_offset + 2] =
                        limb2b.as_felt() + const_expr!(1 << 3) * word.get_felt(read_offset + 3);

                    range_check(
                        ab,
                        &[3, 6, 6, 3],
                        &[limb1a, limb1b.as_felt(), limb2a, limb2b.as_felt()],
                    );
                } else {
                    let limb2a = word.get_felt(read_offset + 2);
                    result[word_offset + write_offset + 1] =
                        limb1b.as_felt() + const_expr!(1 << 6) * limb2a;

                    if word_index == 0 {
                        to_range_check.extend_from_slice(&[limb1a, limb1b.as_felt()]);
                    } else {
                        to_range_check.extend_from_slice(&[limb1b.as_felt(), limb1a]);
                    }
                }
            }
        }

        range_check(ab, &[3, 6, 6, 3], &to_range_check);

        result
    }
}
