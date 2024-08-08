use super::verify_bitwise_xor::*;

use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;

#[derive(Debug)]
pub struct BitwiseXor {
    pub num_bits: usize,
}

// Calculates the bitwise XOR of two Felt expressions.
impl AirFn for BitwiseXor {
    type In = [FeltExpr; 2];
    type Out = FeltExpr;

    fn trace_type(&self) -> TraceType {
        TraceType::Inline
    }

    fn call(&self, air_builder: &mut AirBuilder, [a, b]: Self::In) -> Self::Out {
        let mut a_xor_b = air_builder
            .let_for_deduction(UInt16Expr::from(a.clone()) ^ UInt16Expr::from(b.clone()));
        let a_xor_b = air_builder.deduce(a_xor_b.as_felt_mut());
        air_builder.lookup_call(
            &VerifyBitwiseXor {
                num_bits: self.num_bits,
            },
            [a, b, a_xor_b.clone()],
        );
        a_xor_b
    }
}
