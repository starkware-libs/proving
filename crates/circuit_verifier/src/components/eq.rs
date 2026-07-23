use stwo::core::fields::qm31::SecureField;

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 4;
pub const N_INTERACTION_COLUMNS: usize = 4;

pub struct CircuitEqComponent {}

impl<Value: IValue> CircuitEval<Value> for CircuitEqComponent {
    fn name(&self) -> String {
        "eq".to_string()
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        let m31_gate_relation_id = context.constant(SecureField::from(M31::from(378353459)));
        let in0_address =
            acc.get_preprocessed_column(&PreProcessedColumnId { id: "eq_in0_address".to_owned() });
        let in1_address =
            acc.get_preprocessed_column(&PreProcessedColumnId { id: "eq_in1_address".to_owned() });

        let [in_col0, in_col1, in_col2, in_col3] = *component_data.trace_columns() else {
            panic!("Expected {N_TRACE_COLUMNS} trace columns")
        };

        acc.add_to_relation(
            context,
            context.one(),
            &[m31_gate_relation_id, in0_address, in_col0, in_col1, in_col2, in_col3],
        );
        acc.add_to_relation(
            context,
            context.one(),
            &[m31_gate_relation_id, in1_address, in_col0, in_col1, in_col2, in_col3],
        );
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &[RelationUse { relation_id: "gate", uses: 2 }]
    }

    fn log_size(
        &self,
        preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
    ) -> Option<u32> {
        preprocessed_column_log_sizes
            .get(&PreProcessedColumnId { id: "eq_in0_address".to_string() })
            .cloned()
    }
}
