use inst_def::InstDef;

use super::keys::*;
use crate::airs::casm::const_tables::seq::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252width27_expr::*;
use crate::core::expressions::felt_expr::*;
#[cfg(test)]
use crate::core::variables::*;

/// Lookup component into the table of round keys of Poseidon.
/// The input represents a round number:
///    0..4 correspond to the first 4 full rounds;
///    4..31 correspond to 27 triplets of partial rounds (6-87);
///    31..35 correspond to the last 4 full rounds;
/// The output consists of 30 constant columns, representing 3 Felt252s in Width27 form.
#[derive(Debug, InstDef)]
pub struct PoseidonRoundKeys {}

impl AirFn for PoseidonRoundKeys {
    type ExtIn = SeqConstLen<6>;
    type In = ();
    type Out = [Felt252Width27Expr; 3];

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        [_round_number]: [FeltExpr; 1],
        _: (),
    ) -> Self::Out {
        #[cfg(test)]
        air_builder.set_row_number(_round_number.value().map(|v| v.0 as usize));
        air_builder.call_external_table(&Keys {})
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn deduce_output(&self) -> Option<String> {
        // TODO(DanC): Implement this in stwo-cairo
        Some(format!(
            "{}::deduce_output",
            self.relation_name().expect("Relation name not found")
        ))
    }
}
