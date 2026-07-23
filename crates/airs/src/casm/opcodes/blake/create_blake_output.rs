use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use serde::Serialize;

use super::round::*;
use super::triple_xor32::*;

pub type BlakeH = [UInt32Expr; 8];
#[derive(Debug, Serialize)]
pub struct CreateBlakeOutput {}

/// Creates the BlakeCompress output from `h` (the input array of 8 `u32` elements) and the
/// BlakeState as output by the round.
impl AirFn for CreateBlakeOutput {
    type ExtIn = ();
    type In = (BlakeH, BlakeState);
    type Out = BlakeH;

    fn call(&self, air_builder: &mut AirBuilder, _: (), (h, state): Self::In) -> Self::Out {
        let triple_xor = &TripleXor32 {};

        let mut output = vec![];
        for i in 0..8 {
            output.push(air_builder.lookup_call(
                triple_xor,
                (),
                [state[i].clone(), state[i + 8].clone(), h[i].clone()],
            ));
        }
        output.try_into().expect("Output length should be 8")
    }
}
