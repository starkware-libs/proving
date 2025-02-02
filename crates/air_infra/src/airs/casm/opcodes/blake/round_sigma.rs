use inst_def::InstDef;

use super::sigma::*;
// Macros
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

const STWO_COMPONENT_TYPE_BLAKE_ROUND_NUMBER: &str = "BlakeRoundNumber";

#[derive(Debug, Default, Clone)]
pub struct RoundNumber {}
impl ExtTable for RoundNumber {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_BLAKE_ROUND_NUMBER;
    type T = FeltExpr;
}

/// Lookup component into the constant table Sigma.
/// The input is a constant column with values from 0 to 9, representing the round number of Blake.
/// The output consists of constant columns with a width of 16, containing the message permutation
/// for the corresponding round.
#[derive(Debug, InstDef)]
pub struct RoundSigma {}

impl AirFn for RoundSigma {
    type ExtIn = RoundNumber;
    type In = ();
    type Out = [FeltExpr; 16];

    fn call(&self, air_builder: &mut AirBuilder, _round: FeltExpr, _: ()) -> Self::Out {
        #[cfg(test)]
        air_builder.set_row_number(_round.value().map(|v| v.0 as usize));
        air_builder.call_external_table(&Sigma {})
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
