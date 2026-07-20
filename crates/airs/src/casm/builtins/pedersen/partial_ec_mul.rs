use std::array::from_fn;

use air_common::TraceType;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn, ChainRoundAirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::{ChainIdVar, RoundNumVar};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};

use crate::casm::builtins::ec_utils::ec_add::*;
use crate::casm::builtins::ec_utils::utils::ECPoint;
use crate::casm::builtins::pedersen::points_table::*;

#[derive(Debug, Serialize)]
pub struct PartialECMul<const NUM_WINDOWS: usize> {
    window_bits: usize,
}

impl<const NUM_WINDOWS: usize> PartialECMul<NUM_WINDOWS> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        assert_eq!(252 % NUM_WINDOWS, 0);
        Self { window_bits: 252 / NUM_WINDOWS }
    }
}

const _: () = {
    assert!(
        FELT252_N_WORDS.is_multiple_of(2),
        "PartialECMul {{ 18 }} stores the multiplier as pairs of limbs"
    );
};
pub type PackedECMultiplier<const NUM_WINDOWS: usize> = [FeltExpr; NUM_WINDOWS];
pub type PartialECMulState<const NUM_WINDOWS: usize> = (PackedECMultiplier<NUM_WINDOWS>, ECPoint);

/// Converts a felt252 to the packed limbs format used for the PartialECMul multiplier.
pub fn felt252_to_limbs<const NUM_WINDOWS: usize>(
    value: Felt252Expr,
) -> PackedECMultiplier<NUM_WINDOWS> {
    const HALF_FELT252_N_WORDS: usize = FELT252_N_WORDS / 2;
    match NUM_WINDOWS {
        HALF_FELT252_N_WORDS => from_fn(|i| {
            value.get_felt(i * 2)
                + value.get_felt(i * 2 + 1) * const_expr!(1 << FELT252_BITS_PER_WORD)
        }),
        FELT252_N_WORDS => from_fn(|i| value.get_felt(i)),
        _ => panic!("Unsupported NUM_WINDOWS val {NUM_WINDOWS}"),
    }
}

// Implements the EC partial-mul round relation.
//
// This relation is used to compute values of the form m*P + Q where P,Q are points on the
// STARK curve, and P is one of the high-entropy constant points used in the Pedersen hash.
//
// The computation is done by splitting `m` into 18-bit windows, multiplying P by each part
// and adding the result to an accumulator that was initialized with Q. The multiples of P
// are taken from a precomputed table (PedersenPointsTable).
//
// To avoid having the zero point in the table, each value in the table has P_shift
// subtracted from it. Therefore the result after `w` rounds is shifted by w * P_shift.
// The initial value of Q, drawn from the third section of the PedersenPointsTable, cancels
// out this difference and handles the low-entropy P_1 and P_3 contributions.
//
// The relation is
//    (c, 14 * i + w, m_c >> (w * 18), (m_c)_(w * 18) * P_{2i} + Q_c - w * P_shift),
// where:
// - `c` is the "chain index", used to separate different chains in the component.
// - Q_c is the initial point used in the computation of the chain with index `c`.
// - `14 * i + w` is the round number, ranging from 0 to 27, with i in [0, 2) and w in [0, 14). The
//   high-bit i of the round number indicates the value of P, with i=0 corresponding to P=P_0 and
//   i=1 to P=P_2. The low part w indicates the relative shift of P being added.
// - m_c is the coefficient of P in the chain with index `c`.
// - (m_c)_(w * 18) are the w*18 least-significant bits of m_c.
// The third element (m_c >> (w * 18)) is represented as an array of 18-bit limbs to
// save trace cells.
//
// To use this relation for a multiplication with `k` windows, the caller should
// 1. Yield (c, 14 * i,     m_c, Q_c)
// 2. Use   (c, 14 * i + k, 0,   m_c * P_{2i} + Q_c)
// 3. Add the `k` round rows to this component
impl<const NUM_WINDOWS: usize> AirFn for PartialECMul<NUM_WINDOWS> {
    type ExtIn = ();
    type In = (ChainIdVar, RoundNumVar, PartialECMulState<NUM_WINDOWS>);
    type Out = (ChainIdVar, RoundNumVar, PartialECMulState<NUM_WINDOWS>);

    fn trace_type(&self) -> TraceType {
        TraceType::ChainRound
    }

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        (chain_index, round_index, (m_shifted, accumulator)): Self::In,
    ) -> Self::Out {
        // Shift `m` one window to the right.
        let window_bits = self.window_bits;
        let mut new_m_shifted_elements = m_shifted[1..NUM_WINDOWS].to_vec();
        new_m_shifted_elements.push(const_expr!(0));

        // Read partial product from the PedersenPoints table
        let window = m_shifted[0].clone();
        let rows_per_window = 1 << window_bits;
        let partial_product_location = const_expr!(rows_per_window) * round_index.clone() + window;
        let partial_product = match window_bits {
            9 => {
                let points_table_air = PedersenPointsTable::<15> { window_bits };
                air_builder.lookup_call(&points_table_air, [partial_product_location], ())
            }
            18 => {
                let points_table_air = PedersenPointsTable::<23> { window_bits };
                air_builder.lookup_call(&points_table_air, [partial_product_location], ())
            }
            _ => panic!("Unsupported window_bits value {window_bits}"),
        };

        // Compute output
        let new_accumulator = air_builder.call(
            &ECAdd {},
            [
                accumulator[0].clone(),
                accumulator[1].clone(),
                partial_product[0].clone(),
                partial_product[1].clone(),
            ],
        );

        let new_m_shifted: PackedECMultiplier<NUM_WINDOWS> = new_m_shifted_elements
            .try_into()
            .expect("New m_shifted was built to have NUM_WINDOWS elements");

        (chain_index, round_index + const_expr!(1), (new_m_shifted, new_accumulator))
    }
}

impl<const NUM_WINDOWS: usize> ChainRoundAirFn<PartialECMulState<NUM_WINDOWS>>
    for PartialECMul<NUM_WINDOWS>
{
    fn number_of_chains(&self) -> usize {
        2
    }
}
