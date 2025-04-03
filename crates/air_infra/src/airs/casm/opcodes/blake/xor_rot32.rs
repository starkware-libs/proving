use serde::Serialize;

use crate::airs::casm::bitwise_xor::*;
use crate::airs::casm::opcodes::blake::split16::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;

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
        assert!(
            self.r == 7 || self.r == 8 || self.r == 12 || self.r == 16,
            "Invalid r value"
        );

        // In the case where self.r is 16, we will perform lookups into a table of size 8.
        let r = if self.r == BLAKE_NUM_BITS_PER_FELT || self.r == 0 {
            8
        } else {
            self.r
        };
        // Split into 4 parts of sizes [r, 16-r, r, 16-r].
        let split = Split16 { low_part_size: r };
        let [all, alh] = air_builder.call(&split, a.low());
        let [ahl, ahh] = air_builder.call(&split, a.high());
        let [bll, blh] = air_builder.call(&split, b.low());
        let [bhl, bhh] = air_builder.call(&split, b.high());

        // Calculate and deduce the bitwise xor of the parts.
        let cll = air_builder.call(&BitwiseXor { num_bits: r }, [all, bll]);
        let clh = air_builder.call(
            &BitwiseXor {
                num_bits: BLAKE_NUM_BITS_PER_FELT - r,
            },
            [alh, blh],
        );
        let chl = air_builder.call(&BitwiseXor { num_bits: r }, [ahl, bhl]);
        let chh = air_builder.call(
            &BitwiseXor {
                num_bits: BLAKE_NUM_BITS_PER_FELT - r,
            },
            [ahh, bhh],
        );

        let output = if self.r == BLAKE_NUM_BITS_PER_FELT {
            // For the case self.r=16, we will build the new pair as [chl, chh, cll, clh]
            vec![
                chl + chh * const_expr!(1 << r),
                cll + clh * const_expr!(1 << r),
            ]
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
