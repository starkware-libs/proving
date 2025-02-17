use std::array::from_fn;

use inst_def::InstDef;
use stwo_cairo_common::prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};

use crate::airs::casm::builtins::pedersen::ec_add::*;
use crate::airs::casm::builtins::pedersen::points_table::*;
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

#[derive(Debug, InstDef)]
pub struct PartialECMul {}

const _: () = {
    assert!(
        FELT252_N_WORDS % 2 == 0,
        "PartialECMul stores the multiplier as pairs of limbs"
    );
};
pub type ECPoint = [Felt252Expr; 2];
pub type PackedECMultiplier = [FeltExpr; FELT252_N_WORDS / 2];
pub type PartialECMulState = (FeltExpr, PackedECMultiplier, ECPoint);

/// Convert a felt252 to double-limbs format. This is the format used for the PartialECMul
/// multiplier.
pub fn felt252_to_double_limbs(value: Felt252Expr) -> PackedECMultiplier {
    from_fn(|i| {
        value.get_felt(i * 2) + value.get_felt(i * 2 + 1) * const_expr!(1 << FELT252_BITS_PER_WORD)
    })
}

// Implements the EC partial-mul round relation.
//
// This relation is used to compute values of the form m*P + Q where P,Q are points
// on the STARK curve, and P is one of the constant points used in the Pedersen hash.
//
// The computation is done by splitting `m` into 18-bit windows, multiplying P by each
// part and adding the result to an accumulator that was initialized with Q. The
// multiples of P are taken from a precomputed table (PedersenPointsTable).
//
// To avoid having the zero point in the table, each value in the table has P_shift
// subtracted from it. Therefore the result after `w` rounds is shifted by w * P_shift.
// The caller is responsible for choosing Q appropriately to cancel this difference.
//
// The relation is (c, w, i, m_c >> (w * 18), (m_c)_(w * 18) * P_c + Q_c - w * P_shift), where:
// - `c` is the "chain index", used to separate different chains in the component.
// - P_c, Q_c are the points used in the computatin in the chain with index `c`.
// - `w` is the round number.
// - `i` is the offset from the table start to the part that contains the data for P_c.
// - m_c is the coefficient of P in the chain with index `c`.
// - (m_c)_(w * 18) are the w*18 least-significant bits of m_c.
// The fourth element (m_c >> (w * 18)) is represented as an array of 18-bit limbs to
// save trace cells.
//
// To use this relation for a multiplication with `k` windows, the caller should
// 1. Yield (c, 0, i, m_c, Q_c)
// 2. Use   (c, k, i, 0,   m_c * P_c + Q_c)
// 3. Add the `k` round rows to this component
impl AirFn for PartialECMul {
    type ExtIn = ();
    type In = (ChainIdVar, RoundNumVar, PartialECMulState);
    type Out = (ChainIdVar, RoundNumVar, PartialECMulState);

    fn trace_type(&self) -> TraceType {
        TraceType::ChainRound
    }

    fn call(
        &self,
        air_builder: &mut crate::core::air_fn::AirBuilder,
        _: (),
        (chain_index, round_index, (table_offset, m_shifted, accumulator)): Self::In,
    ) -> Self::Out {
        // Shift `m` 18 bits to the right. We use the fact that Felt252 limbs are 9 bits each,
        // so 18 bits are a single double-limb.
        assert_eq!(BITS_PER_WINDOW, FELT252_BITS_PER_WORD * 2);
        let mut new_m_shifted_elements = m_shifted[1..FELT252_N_WORDS / 2].to_vec();
        new_m_shifted_elements.push(const_expr!(0));

        // Read partial product from the PedersenPoints table
        let window = m_shifted[0].clone();
        let partial_product_location = table_offset.clone()
            + const_expr!(
                <usize as std::convert::TryInto<u32>>::try_into(ROWS_PER_WINDOW).unwrap()
            ) * round_index.clone()
            + window;
        let partial_product =
            air_builder.lookup_call(&PedersenPointsTable {}, [partial_product_location], ());

        // Compute output
        let new_m_shifted: PackedECMultiplier = new_m_shifted_elements
            .try_into()
            .expect("New m_shifted was built to have FELT252_N_WORDS / 2 elements");
        let new_accumulator = air_builder.call(
            &ECAdd {},
            [
                accumulator[0].clone(),
                accumulator[1].clone(),
                partial_product[0].clone(),
                partial_product[1].clone(),
            ],
        );
        (
            chain_index,
            round_index + const_expr!(1),
            (table_offset, new_m_shifted, new_accumulator),
        )
    }

    fn deduce_output(&self) -> Option<String> {
        // TODO(adar): Implement this in stwo-cairo
        Some(format!(
            "{}::deduce_output",
            self.relation_name().expect("Relation name not found")
        ))
    }
}

impl ChainRoundAirFn<PartialECMulState> for PartialECMul {
    fn number_of_chains(&self) -> usize {
        4
    }
}
