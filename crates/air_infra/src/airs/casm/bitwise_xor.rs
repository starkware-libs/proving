use inst_def::InstDef;

use super::const_tables::verify_bitwise_xor::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;

#[derive(Debug, InstDef)]
pub struct BitwiseXor {
    pub num_bits: usize,
}

// Calculates the bitwise XOR of two Felt expressions.
impl AirFn for BitwiseXor {
    type ExtIn = ();
    type In = [FeltExpr; 2];
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b]: Self::In) -> Self::Out {
        let mut a_xor_b = air_builder.let_for_deduction(
            UInt16Expr::from(a.clone()) ^ UInt16Expr::from(b.clone()),
            "xor",
        );
        let a_xor_b = air_builder.deduce(a_xor_b.as_felt_mut(), "xor");

        match self.num_bits {
            4 => air_builder.lookup_call(
                &VerifyBitwiseXor::<VerifyBitwiseXor4>::default(),
                [a, b, a_xor_b.clone()],
                (),
            ),
            7 => air_builder.lookup_call(
                &VerifyBitwiseXor::<VerifyBitwiseXor7>::default(),
                [a, b, a_xor_b.clone()],
                (),
            ),
            8 => air_builder.lookup_call(
                &VerifyBitwiseXor::<VerifyBitwiseXor8>::default(),
                [a, b, a_xor_b.clone()],
                (),
            ),
            9 => air_builder.lookup_call(
                &VerifyBitwiseXor::<VerifyBitwiseXor9>::default(),
                [a, b, a_xor_b.clone()],
                (),
            ),
            12 => air_builder.lookup_call(
                &VerifyBitwiseXor::<VerifyBitwiseXor12>::default(),
                [a, b, a_xor_b.clone()],
                (),
            ),
            _ => panic!("Unsupported number of bits: {}", self.num_bits),
        }
        a_xor_b
    }
}
