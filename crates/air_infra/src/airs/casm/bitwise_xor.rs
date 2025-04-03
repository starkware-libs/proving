use serde::Serialize;

use super::const_tables::verify_bitwise_xor::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;

#[derive(Debug, Serialize)]
pub struct BitwiseXor {
    pub num_bits: usize,
}

// Calculates the bitwise XOR of two Felt expressions.
impl AirFn for BitwiseXor {
    type ExtIn = ();
    type In = [FeltExpr; 2];
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b]: Self::In) -> Self::Out {
        let a_xor_b = air_builder.deduce_air_var(
            UInt16Expr::from(a.clone()) ^ UInt16Expr::from(b.clone()),
            "xor",
        );
        verify_bitwise_xor(air_builder, self.num_bits as u16, [a, b, a_xor_b.as_felt()]);
        a_xor_b.as_felt()
    }
}
