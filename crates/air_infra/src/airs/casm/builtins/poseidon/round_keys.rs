use inst_def::InstDef;

use super::keys::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252packed27_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

const STWO_COMPONENT_TYPE_POSEIDON_ROUND_NUMBER: &str = "PoseidonRoundNumber";

#[derive(Debug, Default, Clone)]
pub struct RoundNumber {}
impl ExtTable for RoundNumber {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_POSEIDON_ROUND_NUMBER;
    type T = FeltExpr;
}

/// Lookup component into the table of round keys of Poseidon.
/// The input represents a round number:
///    0..4 correspond to the first 4 full rounds;
///    4..31 correspond to 27 triplets of partial rounds (6-87);
///    31..35 correspond to the last 4 full rounds;
/// The output consists of 30 constant columns, representing 3 Felt252s in Packed27 form.
#[derive(Debug, InstDef)]
pub struct PoseidonRoundKeys {}

impl AirFn for PoseidonRoundKeys {
    type ExtIn = RoundNumber;
    type In = ();
    type Out = [Felt252Packed27Expr; 3];

    fn call(&self, air_builder: &mut AirBuilder, _round_number: FeltExpr, _: ()) -> Self::Out {
        #[cfg(test)]
        air_builder.set_row_number(_round_number.value().map(|v| v.0 as usize));
        air_builder.call_external_table(&Keys {})
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
