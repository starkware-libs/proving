use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use serde::Serialize;

use crate::casm::bitwise_xor::bitwise_xor::*;
use crate::casm::bitwise_xor::verify_bitwise_xor::*;
use crate::casm::opcodes::blake::split16::*;

const BLAKE_NUM_BITS_PER_FELT: usize = 16;

#[derive(Debug, Serialize)]
pub struct XorRot32 {
    pub r: usize,
}

/// Receives a and b as UInt32 which is a pair of felts - low and high.
/// Computes their XOR rotated right by `r` bits by splitting each one into 4 parts of sizes [r,
/// 16-r, r, 16-r], performing 4 lookups into XOR constant tables of corresponding sizes [r, 16-r,
/// r, 16-r], and constructing the new UInt32 according to the rotation, such that the lowest part
/// of size `r` becomes the highest part. Ensures that all elements are within range.
/// 'r' must be one of 7, 8, 12, 16.
impl AirFn for XorRot32 {
    type ExtIn = ();
    type In = [UInt32Expr; 2];
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b]: Self::In) -> Self::Out {
        assert!(self.r == 7 || self.r == 8 || self.r == 12 || self.r == 16, "Invalid r value");

        // In the case where self.r is 16, we will perform lookups into a table of size 8.
        let r = if self.r == BLAKE_NUM_BITS_PER_FELT || self.r == 0 { 8 } else { self.r };
        let variant = (r == 8 && BLAKE_NUM_BITS_PER_FELT == 16) as usize;

        // Split into 4 parts of sizes [r, 16-r, r, 16-r].
        let split = Split16 { low_part_size: r };
        let [all, alh] = air_builder.call(&split, a.low());
        let [ahl, ahh] = air_builder.call(&split, a.high());
        let [bll, blh] = air_builder.call(&split, b.low());
        let [bhl, bhh] = air_builder.call(&split, b.high());

        // Calculate and deduce the bitwise xor of the parts.
        let cll = air_builder.call(&BitwiseXor { num_bits: r, variant: 0 }, [all, bll]);
        let clh = air_builder
            .call(&BitwiseXor { num_bits: BLAKE_NUM_BITS_PER_FELT - r, variant: 0 }, [alh, blh]);
        let chl = air_builder.call(&BitwiseXor { num_bits: r, variant }, [ahl, bhl]);
        let chh = air_builder
            .call(&BitwiseXor { num_bits: BLAKE_NUM_BITS_PER_FELT - r, variant }, [ahh, bhh]);

        let output = if self.r == BLAKE_NUM_BITS_PER_FELT {
            // For the case self.r=16, we will build the new pair as [chl, chh, cll, clh]
            vec![chl + chh * const_expr!(1 << r), cll + clh * const_expr!(1 << r)]
        } else {
            // For the other cases, we will build the new pair as [clh, chl, chh, cll]
            vec![
                clh + chl * const_expr!(1 << (BLAKE_NUM_BITS_PER_FELT - r)),
                chh + cll * const_expr!(1 << (BLAKE_NUM_BITS_PER_FELT - r)),
            ]
        };

        air_builder.let_(output.into(), &format!("xor_rot_{}_output", self.r))
    }
}

#[derive(Debug, Serialize)]
pub struct VerifyXorRot32 {
    pub r: usize,
}

/// Verifies that `c` equals XOR of `a` and `b` rotated right by `r` bits, where `a`, `b`, and
/// `c` are UInt32 values represented as pairs of felts (low, high).
/// Splits `a` and `b` into 4 parts of sizes [r, 16-r, r, 16-r], splits `c` with the opposite
/// sizes [16-r, r, 16-r, r] to recover the XOR parts in their rotated positions, then performs 4
/// `verify_bitwise_xor` lookups to confirm correctness without deducing the result.
impl AirFn for VerifyXorRot32 {
    type ExtIn = ();
    type In = [UInt32Expr; 3];
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b, c]: Self::In) -> Self::Out {
        assert!(self.r == 7 || self.r == 8, "VerifyXorRot32 currently supports r=7 and r=8");

        // Split a and b into 4 parts of sizes [r, 16-r, r, 16-r].
        let split_ab = Split16 { low_part_size: self.r };
        let [all, alh] = air_builder.call(&split_ab, a.low());
        let [ahl, ahh] = air_builder.call(&split_ab, a.high());
        let [bll, blh] = air_builder.call(&split_ab, b.low());
        let [bhl, bhh] = air_builder.call(&split_ab, b.high());

        // Split c with low parts of size (16-r).
        let split_c = Split16 { low_part_size: BLAKE_NUM_BITS_PER_FELT - self.r };
        let [cll, clh] = air_builder.call(&split_c, c.low());
        let [chl, chh] = air_builder.call(&split_c, c.high());

        // Verify that c equals (a XOR b) right-rotated by r, by checking XOR on each pair of
        // sub-limbs. For example, all ^ bll -> chh (low r bits of a^b wrap to high bits of
        // c after right-rotation by r).
        verify_bitwise_xor(
            air_builder,
            (BLAKE_NUM_BITS_PER_FELT - self.r) as u16,
            [alh, blh, cll],
            0,
        );
        verify_bitwise_xor(air_builder, self.r as u16, [ahl, bhl, clh], 0);
        verify_bitwise_xor(
            air_builder,
            (BLAKE_NUM_BITS_PER_FELT - self.r) as u16,
            [ahh, bhh, chl],
            0,
        );
        verify_bitwise_xor(air_builder, self.r as u16, [all, bll, chh], 0);
    }
}
