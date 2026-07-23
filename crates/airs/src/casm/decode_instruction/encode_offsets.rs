use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::range_check::range_check;
use air_infra::{const_expr, const_u16_expr};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_BITS_PER_WORD;

#[derive(Clone, Debug, Serialize)]
pub struct EncodeOffsets {}

// | - |   9   | 9 |   9   | 9 |   9   | 9 | Felts in the instruction
// | - | - | 3 | 9 | 4 | 5 | 9 | 2 | 7 | 9 | Parts of offsets and flags
// |  -    |    16     |    16     |  16   | Offsets
//
// Deduces and range checks the parts of each offset (the range check is only on parts smaller than
// 9 bits, since the 9 bits felts are range checked in memory).
// Reconstructs the offsets and verifies them against the input.
// Constructs the six felts holding the offsets in the instruction.
impl AirFn for EncodeOffsets {
    type ExtIn = ();
    type In = [FeltExpr; 3];
    type Out = [FeltExpr; 6];

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("offset0".to_string()),
            Some("offset1".to_string()),
            Some("offset2".to_string()),
        ])
    }

    fn call(&self, ab: &mut AirBuilder, _: (), [off0_f, off1_f, off2_f]: Self::In) -> Self::Out {
        assert_eq!(
            FELT252_BITS_PER_WORD, 9,
            "OffsetsToFelts assumes there are 9 bits per felt in a felt252"
        );

        // Deduce the parts of offset0.
        let off0 = UInt16Expr::from(off0_f.clone());
        let low0 = ab.deduce_air_var(off0.clone() & const_u16_expr!(0x1FF), "offset0_low");
        let mid0 = ab.deduce_air_var(off0 >> const_u16_expr!(9), "offset0_mid");

        // Reconstruct offset0 as felt from the middle and low parts.
        let new_off0_f = low0.as_felt() + (mid0.as_felt() * const_expr!(1 << 9));
        ab.constrain(new_off0_f - off0_f, "Reconstructed offset0 is correct");

        // Deduce the parts of offset1.
        let off1 = UInt16Expr::from(off1_f.clone());
        let low1 = ab.deduce_air_var(off1.clone() & const_u16_expr!(0x3), "offset1_low");
        let mid1 = ab.deduce_air_var(
            (off1.clone() >> const_u16_expr!(2)) & const_u16_expr!(0x1FF),
            "offset1_mid",
        );
        let high1 = ab.deduce_air_var(off1 >> const_u16_expr!(11), "offset1_high");

        // Reconstruct offset1 as felt from the high, middle and low parts.
        let new_off1_f = (low1.as_felt() + (mid1.as_felt() * const_expr!(1 << 2)))
            + (high1.as_felt() * const_expr!(1 << 11));
        ab.constrain(new_off1_f - off1_f, "Reconstructed offset1 is correct");

        // Deduce the parts of offset2.
        let off2 = UInt16Expr::from(off2_f.clone());
        let low2 = ab.deduce_air_var(off2.clone() & const_u16_expr!(0xF), "offset2_low");
        let mid2 = ab.deduce_air_var(
            (off2.clone() >> const_u16_expr!(4)) & const_u16_expr!(0x1FF),
            "offset2_mid",
        );
        let high2 = ab.deduce_air_var(off2 >> const_u16_expr!(13), "offset2_high");

        // Reconstruct offset2 as felt from the high, middle and low parts.
        let new_off2_f = (low2.as_felt() + (mid2.as_felt() * const_expr!(1 << 4)))
            + (high2.as_felt() * const_expr!(1 << 13));
        ab.constrain(new_off2_f - off2_f, "Reconstructed offset2 is correct");

        range_check(ab, &[7, 2, 5], &[mid0.as_felt(), low1.as_felt(), high1.as_felt()]);
        range_check(ab, &[4, 3], &[low2.as_felt(), high2.as_felt()]);

        let felt0 = low0.as_felt();
        let felt1 = mid0.as_felt() + (low1.as_felt() * const_expr!(1 << 7));
        let felt2 = mid1.as_felt();
        let felt3 = high1.as_felt() + (low2.as_felt() * const_expr!(1 << 5));
        let felt4 = mid2.as_felt();
        let felt5 = high2.as_felt();

        [felt0, felt1, felt2, felt3, felt4, felt5]
    }
}
