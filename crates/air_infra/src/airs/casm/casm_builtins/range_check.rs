use indexmap::IndexMap;

use crate::airs::casm::const_tables::seq::*;
use crate::airs::casm::read_small_felt252::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

use crate::const_expr;

// Start address of the segment for this builtin.
// TODO: receive this at proof time as a public param. Until public params
// are implemented, have it as a dummy constant for testing.
pub const DUMMY_SEGMENT_START: u32 = 100;

#[derive(Debug)]
pub struct RangeCheckBuiltin {
    pub bits: usize,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for RangeCheckBuiltin {
    type In = ();
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _input: Self::In) -> Self::Out {
        let instance_number = air_builder.call_external_column(&Seq {});
        air_builder.call(
            &ReadSmallFelt252 {
                num_bits: self.bits,
                memory: self.memory.clone(),
            },
            const_expr!(DUMMY_SEGMENT_START) + instance_number,
        );
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        [("bits".to_string(), self.bits.to_string())].into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
