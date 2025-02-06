use inst_def::InstDef;

use super::triple_sum32::*;
use super::xor_rot32::*;
use crate::core::air_fn::*;
use crate::core::expressions::uint32_expr::*;

const NUM_INPUT_WORDS_G: usize = 6;
const NUM_OUTPUT_WORDS_G: usize = 4;

// Macros
use crate::const_u32_expr;

#[derive(Debug, InstDef)]
pub struct BlakeG {}

impl AirFn for BlakeG {
    type ExtIn = ();
    type In = [UInt32Expr; NUM_INPUT_WORDS_G];
    type Out = [UInt32Expr; NUM_OUTPUT_WORDS_G];

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        [a, b, c, d, f0, f1]: Self::In,
    ) -> Self::Out {
        let a_tmp = air_builder.call(&TripleSum32 {}, [a, b.clone(), f0]);
        let d_tmp = air_builder.call(&XorRot32 { r: 16 }, [a_tmp.clone(), d]);
        let c_tmp = air_builder.call(&TripleSum32 {}, [c, d_tmp.clone(), const_u32_expr!(0)]);
        let b_tmp = air_builder.call(&XorRot32 { r: 12 }, [b, c_tmp.clone()]);
        let a_out = air_builder.call(&TripleSum32 {}, [a_tmp, b_tmp.clone(), f1]);
        let d_out = air_builder.call(&XorRot32 { r: 8 }, [a_out.clone(), d_tmp]);
        let c_out = air_builder.call(&TripleSum32 {}, [c_tmp, d_out.clone(), const_u32_expr!(0)]);
        let b_out = air_builder.call(&XorRot32 { r: 7 }, [b_tmp, c_out.clone()]);
        [a_out, b_out, c_out, d_out]
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn deduce_output(&self) -> Option<String> {
        // TODO(Stav): Implement this in stwo-cairo
        Some(format!(
            "{}::deduce_output",
            self.relation_name().expect("Relation name not found")
        ))
    }
}
