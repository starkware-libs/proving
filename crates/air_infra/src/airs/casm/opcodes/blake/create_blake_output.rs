use inst_def::InstDef;

use super::round::*;
use super::triple_xor32::*;
// Macros
use crate::core::air_fn::*;
use crate::core::expressions::uint32_expr::*;

pub type H = [UInt32Expr; 8];
#[derive(Debug, InstDef)]
pub struct CreateBlakeOutput {}

/// Creates the BlakeCompress output from `h` (the input array of 8 `u32` elements) and the
/// BlakeState as output by the round.
impl AirFn for CreateBlakeOutput {
    type ExtIn = ();
    type In = (H, BlakeState);
    type Out = H;

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
