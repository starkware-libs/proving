use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use serde::Serialize;

use super::verify_bitwise_xor::*;

#[derive(Debug, Serialize)]
pub struct BitwiseXor {
    pub num_bits: usize,
    pub variant: usize,
}

impl BitwiseXor {
    pub fn new(num_bits: usize) -> Self {
        Self { num_bits, variant: 0 }
    }
}

// Calculates the bitwise XOR of two Felt expressions.
impl AirFn for BitwiseXor {
    type ExtIn = ();
    type In = [FeltExpr; 2];
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b]: Self::In) -> Self::Out {
        let a_xor_b = air_builder
            .deduce_air_var(UInt16Expr::from(a.clone()) ^ UInt16Expr::from(b.clone()), "xor");
        verify_bitwise_xor(
            air_builder,
            self.num_bits as u16,
            [a, b, a_xor_b.as_felt()],
            self.variant,
        );
        a_xor_b.as_felt()
    }

    fn name(&self) -> String {
        match self.variant {
            0 => format!("bitwise_xor_num_bits_{}", self.num_bits),
            _ => format!("bitwise_xor_num_bits_{}_b", self.num_bits),
        }
    }
}
