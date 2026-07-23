use std::array::from_fn;

use air_common::TraceType;
use air_infra::core::air_fn::{AirFn, ChainRoundAirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252width27_expr::Felt252Width27Expr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::core::variables::{AirVar, ChainIdVar, RoundNumVar};
use air_infra::{const_expr, const_u16_expr};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::{FELT252_N_WORDS, FELT252WIDTH27_N_WORDS};

use crate::casm::builtins::ec_utils::ec_add::*;
use crate::casm::builtins::ec_utils::ec_double::*;
use crate::casm::builtins::ec_utils::utils::ECPoint;
use crate::felt252_utils::verify_reduced252::*;

#[derive(Debug, Serialize)]
pub struct PartialECMulGeneric {}

pub type PartialECMulGenericState = (Felt252Width27Expr, ECPoint, ECPoint, FeltExpr);

// Implements the generic EC partial-mul round relation.
//
// This relation is used to compute values of the form P + m*Q where P,Q are points on the
// STARK curve.
//
// The relation is
//    (c, r, m_c >> r, Q_c << r, P_c + (m_c)_r * Q_c, counter),
// where:
// - `c` is the "chain index", used to separate different chains in the component.
// - P_c, Q_c are inputs used in the computation of the chain with index `c`.
// - `r` is the round number, ranging from 0 to 251.
// - m_c is the coefficient of Q_c in the chain with index `c`.
// - (m_c)_r are the r least-significant bits of m_c.
// - `counter` is an auxiliary term equal to (26 - r) % 27, used for determining the round type.
// The third element m_c >> r is represented as an array of 27-bit limbs to save trace cells.
// The shift is performed by shifting the entire array [r/27] limbs, and the least limb by r % 27.
//
// To use this relation for a multiplication with `k` rounds, the caller should
// 1. Yield (c, 0, m_c, Q_c,      P_c,             26           )
// 2. Use   (c, k, 0,   Q_c << k, P_c + m_c * Q_c, (26 - k) % 27)
// 3. Add the `k` round rows to this component
// 4. Important: If k % 27 != 0, the caller must constrain that the final m_c-s least limb indeed
//    equals 0, to guarantee soundness of the final (k % 27) rounds.
impl AirFn for PartialECMulGeneric {
    type ExtIn = ();
    type In = (ChainIdVar, RoundNumVar, PartialECMulGenericState);
    type Out = (ChainIdVar, RoundNumVar, PartialECMulGenericState);

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("chain_id".to_string()),
            Some("round_num".to_string()),
            Some("m".to_string()),
            Some("q_x".to_string()),
            Some("q_y".to_string()),
            Some("accumulator_x".to_string()),
            Some("accumulator_y".to_string()),
            Some("counter".to_string()),
        ])
    }

    fn trace_type(&self) -> TraceType {
        TraceType::ChainRound
    }

    fn call(
        &self,
        air_builder: &mut air_infra::core::air_fn::AirBuilder,
        _: (),
        (chain_index, round_index, (m, q, accumulator, counter)): Self::In,
    ) -> Self::Out {
        // Get to_add_bit = m & 1.
        let low_limb = air_builder.let_for_deduction(UInt32Expr::from(m.get_felt(0)), "m0");
        let to_add_bit =
            air_builder.let_for_deduction(low_limb.low() & const_u16_expr!(1), "to_add_bit");
        let to_add_bit = air_builder.deduce_air_var(to_add_bit.as_felt(), "to_add_bit");
        air_builder.constrain(
            to_add_bit.clone() * (const_expr!(1) - to_add_bit.clone()),
            "to_add_bit is bool",
        );

        // TODO(DanC): Consider refactoring to use 1 fewer column (but more constraints).

        // Evaluate and constrain is_special_round = (counter == 0).
        // We enforce this by deducing counter_inverse, which equals 1/counter if counter != 0 and
        // 1 if counter == 0, and constraining `counter * counter_inverse = 1 - is_special round`.
        // is_special_round is constrained to be boolean, so this implies that either
        //  (1) is_special_round = 0, counter != 0 (counter_inverse = 1 / counter)
        //  (2) is_special_round = 1, counter = 0 (counter_inverse is unconstrained)
        //  (3) is_special_round = 1, counter_inverse = 0 (counter is unconstrained)
        // Note that counter_inverse is in fact always non-zero, which we enforce by constraining
        // `counter_inverse * (counter + is_special_round) = 1`.
        // This rules out case (3), and the remaining cases match is_special_round = (counter == 0).
        let is_special_round =
            air_builder.let_for_deduction(counter.clone().eq(const_expr!(0)), "is_special_round");
        let is_special_round =
            air_builder.deduce_air_var(is_special_round.as_felt(), "is_special_round");
        let not_is_special_round =
            air_builder.let_(const_expr!(1) - is_special_round.clone(), "not_is_special_round");
        let counter_inverse_inverse =
            air_builder.let_(counter.clone() + is_special_round.clone(), "counter_inverse_inverse");
        let counter_inverse = air_builder
            .deduce_air_var(counter_inverse_inverse.clone().inverse(), "counter_inverse");
        air_builder.constrain(
            is_special_round.clone() * not_is_special_round.clone(),
            "is_special_round is bool",
        );
        air_builder.constrain(
            counter.clone() * counter_inverse.clone() - not_is_special_round.clone(),
            "is_special_round = (counter == 0)",
        );
        air_builder.constrain(
            counter_inverse * counter_inverse_inverse - const_expr!(1),
            "counter_inverse != 0",
        );

        // Compute next_m and constrain according to is_special_round.
        let m0_minus_to_add_bit =
            air_builder.let_(m.get_felt(0) - to_add_bit.clone(), "m0_minus_to_add_bit");
        air_builder.constrain(
            m0_minus_to_add_bit.clone() * is_special_round,
            "m0 is exhausted at the end of special rounds",
        );
        let mut next_m_vec = Vec::new();
        next_m_vec.push(air_builder.assign(
            &mut mux(
                not_is_special_round.clone(),
                m.get_felt(1),
                m0_minus_to_add_bit / const_expr!(2),
            ),
            "next_m_0",
        ));
        for i in 1..(FELT252WIDTH27_N_WORDS - 1) {
            next_m_vec.push(air_builder.assign(
                &mut mux(not_is_special_round.clone(), m.get_felt(i + 1), m.get_felt(i)),
                &format!("next_m_{i}"),
            ));
        }
        let i = FELT252WIDTH27_N_WORDS - 1;
        next_m_vec.push(
            air_builder.assign(
                &mut (m.get_felt(i) * not_is_special_round.clone()),
                &format!("next_m_{i}"),
            ),
        );
        let next_m = next_m_vec.into();

        // Compute next_counter according to is_special_round.
        let next_counter = air_builder.assign(
            &mut mux(not_is_special_round, const_expr!(26), counter - const_expr!(1)),
            "next_counter",
        );

        // Constrain accumulator.x != q.x, to ensure soundness of ECAdd.
        // This comparison has to be done modulo P. To avoid fully computing the difference
        // modulo P, we instead assert that the two numbers are reduced and not exactly equal.
        air_builder.call(&VerifyReduced252 {}, accumulator[0].clone());
        air_builder.call(&VerifyReduced252 {}, q[0].clone());
        let diff_sum_squares: FeltExpr = q[0]
            .as_felts()
            .into_iter()
            .zip(accumulator[0].as_felts())
            .map(|(x, y)| {
                let d = air_builder.let_(x - y, "q_acc_diff");
                d.clone() * d
            })
            .sum();
        let diff_sum_squares_inv =
            air_builder.deduce(&mut diff_sum_squares.clone().inverse(), "diff_sum_squares_inv");
        air_builder.constrain(
            diff_sum_squares * diff_sum_squares_inv - const_expr!(1),
            "accumulator.x doesn't equal q.x",
        );

        // Compute new_accumulator and double_q.
        let accumulator_with_add = air_builder.call(
            &ECAdd {},
            [accumulator[0].clone(), accumulator[1].clone(), q[0].clone(), q[1].clone()],
        );
        let new_accumulator: ECPoint = from_fn(|j| {
            (0..FELT252_N_WORDS)
                .map(|i| {
                    air_builder.assign(
                        &mut mux(
                            to_add_bit.clone(),
                            accumulator[j].get_felt(i),
                            accumulator_with_add[j].get_felt(i),
                        ),
                        &format!("new_acculumator_{j}_{i}"),
                    )
                })
                .collect::<Vec<_>>()
                .into()
        });
        let double_q = air_builder.call(&ECDouble {}, q);

        (
            chain_index,
            round_index + const_expr!(1),
            (next_m, double_q, new_accumulator, next_counter),
        )
    }
}

impl ChainRoundAirFn<PartialECMulGenericState> for PartialECMulGeneric {
    fn number_of_chains(&self) -> usize {
        1
    }
}

fn mux(selector: FeltExpr, val_0: FeltExpr, val_1: FeltExpr) -> FeltExpr {
    (val_1 - val_0.clone()) * selector + val_0
}
