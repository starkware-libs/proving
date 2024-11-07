use inst_def::InstDef;
use prover_types::cpu::FELT252_BITS_PER_WORD;

use super::super::const_tables::range_check::*;
use crate::airs::felt252_id_memory::memory::*;
// Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;

#[derive(Clone, Debug, InstDef)]
pub struct EncodeOffsets {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

// | - |   9   | 9 |   9   | 9 |   9   | 9 | Felts in the instruction
// | - | - | 3 | 9 | 4 | 5 | 9 | 2 | 7 | 9 | Parts of offsets and flags
// |  -    |    16     |    16     |  16   | Offsets
//
// Deduces and range checks the parts of each offset (the range check is only on parts smaller than
// 9 bits, since the 9 bits felts are range checked in memory).
// Reconstructs the offsets and verifies them against the input.
// Constructs the six felts holding the offsets in the instruction.
impl AirFn for EncodeOffsets {
    type In = [FeltExpr; 3];
    type Out = [FeltExpr; 6];

    fn call(&self, ab: &mut AirBuilder, [off0_f, off1_f, off2_f]: Self::In) -> Self::Out {
        assert_eq!(
            FELT252_BITS_PER_WORD, 9,
            "OffsetsToFelts assumes there are 9 bits per felt in a felt252"
        );

        // Deduce the parts of offset0.
        let off0 = UInt16Expr::from(off0_f.clone());
        let mut low0 = ab.let_for_deduction(off0.clone() & const_u16_expr!(0x1FF), "offset0_low");
        let low0_f = ab.deduce(low0.as_felt_mut(), "offset0_low");
        let mut mid0 = ab.let_for_deduction(off0 >> const_u16_expr!(9), "offset0_mid");
        let mid0_f = ab.deduce(mid0.as_felt_mut(), "offset0_mid");

        // Reconstruct offset0 as felt from the middle and low parts.
        let new_off0_f = low0_f.clone() + (mid0_f.clone() * const_expr!(1 << 9));
        ab.constrain(new_off0_f - off0_f, "Reconstructed offset0 is correct");

        // Deduce the parts of offset1.
        let off1 = UInt16Expr::from(off1_f.clone());
        let mut low1 = ab.let_for_deduction(off1.clone() & const_u16_expr!(0x3), "offset1_low");
        let low1_f = ab.deduce(low1.as_felt_mut(), "offset1_low");
        let mut mid1 = ab.let_for_deduction(
            (off1.clone() >> const_u16_expr!(2)) & const_u16_expr!(0x1FF),
            "offset1_mid",
        );
        let mid1_f = ab.deduce(mid1.as_felt_mut(), "offset1_mid");
        let mut high1 = ab.let_for_deduction(off1 >> const_u16_expr!(11), "offset1_high");
        let high1_f = ab.deduce(high1.as_felt_mut(), "offset1_high");

        // Reconstruct offset1 as felt from the high, middle and low parts.
        let new_off1_f = (low1_f.clone() + (mid1_f.clone() * const_expr!(1 << 2)))
            + (high1_f.clone() * const_expr!(1 << 11));
        ab.constrain(new_off1_f - off1_f, "Reconstructed offset1 is correct");

        // Deduce the parts of offset2.
        let off2 = UInt16Expr::from(off2_f.clone());
        let mut low2 = ab.let_for_deduction(off2.clone() & const_u16_expr!(0xF), "offset2_low");
        let low2_f = ab.deduce(low2.as_felt_mut(), "offset2_low");
        let mut mid2 = ab.let_for_deduction(
            (off2.clone() >> const_u16_expr!(4)) & const_u16_expr!(0x1FF),
            "offset2_mid",
        );
        let mid2_f = ab.deduce(mid2.as_felt_mut(), "offset2_mid");
        let mut high2 = ab.let_for_deduction(off2 >> const_u16_expr!(13), "offset2_high");
        let high2_f = ab.deduce(high2.as_felt_mut(), "offset2_high");

        // Reconstruct offset2 as felt from the high, middle and low parts.
        let new_off2_f = (low2_f.clone() + (mid2_f.clone() * const_expr!(1 << 4)))
            + (high2_f.clone() * const_expr!(1 << 13));
        ab.constrain(new_off2_f - off2_f, "Reconstructed offset2 is correct");

        ab.lookup_call(
            &RangeCheck { bits: [7, 2, 5] },
            [mid0_f.clone(), low1_f.clone(), high1_f.clone()],
        );
        ab.lookup_call(
            &RangeCheck { bits: [4, 3] },
            [low2_f.clone(), high2_f.clone()],
        );

        let felt0 = low0_f;
        let felt1 = mid0_f + (low1_f * const_expr!(1 << 7));
        let felt2 = mid1_f;
        let felt3 = high1_f + (low2_f * const_expr!(1 << 5));
        let felt4 = mid2_f;
        let felt5 = high2_f;

        [felt0, felt1, felt2, felt3, felt4, felt5]
    }
}
