use inst_def::InstDef;

use crate::airs::casm::bitwise_xor::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::expressions::uint32_expr::*;
// Macros
use crate::{const_expr, const_u16_expr};

const BLAKE_NUM_BITS_PER_FELT: usize = 16;

#[derive(Debug, InstDef)]
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
    type In = [UInt32Expr; 2];
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, [a, b]: Self::In) -> Self::Out {
        assert!(
            self.r == 7 || self.r == 8 || self.r == 12 || self.r == 16,
            "Invalid r value"
        );

        // For the case r=16, we will perform lookups into a table of size 8.
        let r = if self.r == BLAKE_NUM_BITS_PER_FELT {
            8
        } else {
            self.r
        };
        let r_expr = const_u16_expr!(r as u16);

        // Calculate and deduce the high 16-r bits for each felt.
        let mut alh = air_builder.let_for_deduction(
            a.low() >> r_expr.clone(),
            &format!("a_low_{}_ms_bits", 16 - r),
        );
        let mut ahh = air_builder.let_for_deduction(
            a.high() >> r_expr.clone(),
            &format!("a_high_{}_ms_bits", 16 - r),
        );
        let mut blh = air_builder.let_for_deduction(
            b.low() >> r_expr.clone(),
            &format!("b_low_{}_ms_bits", 16 - r),
        );
        let mut bhh = air_builder.let_for_deduction(
            b.high() >> r_expr.clone(),
            &format!("b_high_{}_ms_bits", 16 - r),
        );

        air_builder.deduce(alh.as_felt_mut(), &format!("a_low_{}_ms_bits", 16 - r));
        air_builder.deduce(ahh.as_felt_mut(), &format!("a_high_{}_ms_bits", 16 - r));
        air_builder.deduce(blh.as_felt_mut(), &format!("b_low_{}_ms_bits", 16 - r));
        air_builder.deduce(bhh.as_felt_mut(), &format!("b_high_{}_ms_bits", 16 - r));

        // Caclulate the low r bits for each felt.
        let all = a.low().as_felt() - alh.as_felt() * const_expr!(1 << r);
        let bll = b.low().as_felt() - blh.as_felt() * const_expr!(1 << r);
        let ahl = a.high().as_felt() - ahh.as_felt() * const_expr!(1 << r);
        let bhl = b.high().as_felt() - bhh.as_felt() * const_expr!(1 << r);

        // Calculate and deduce the bitwise xor of the parts.
        let cll = air_builder.call(&BitwiseXor { num_bits: r }, [all, bll]);
        let clh = air_builder.call(
            &BitwiseXor {
                num_bits: BLAKE_NUM_BITS_PER_FELT - r,
            },
            [alh.as_felt(), blh.as_felt()],
        );
        let chl = air_builder.call(&BitwiseXor { num_bits: r }, [ahl, bhl]);
        let chh = air_builder.call(
            &BitwiseXor {
                num_bits: BLAKE_NUM_BITS_PER_FELT - r,
            },
            [ahh.as_felt(), bhh.as_felt()],
        );

        let output = if self.r == BLAKE_NUM_BITS_PER_FELT {
            // For the case r=16, we will build the new pair as [chl, chh, cll, clh]
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

        air_builder.let_vec(output, &format!("xor_rot_{}_output", self.r))
    }
}
