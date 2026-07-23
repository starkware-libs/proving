use crate::components::prelude::*;

const MEMORY_ADDRESS_TO_ID_SPLIT: usize = 16;
pub const N_TRACE_COLUMNS: usize = MEMORY_ADDRESS_TO_ID_SPLIT * 2;
pub const N_INTERACTION_COLUMNS: usize = MEMORY_ADDRESS_TO_ID_SPLIT.div_ceil(2) * 4;
pub const MEMORY_ADDRESS_TO_ID_RELATION_ID: u32 = 1444891767;

pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    // Addresses are offsetted by 1, as 0 address is reserved.
    let seq = seq_of_component_size(context, component_data, &acc.preprocessed_columns);
    let mut address = eval!(context, (seq) + (1));

    for i in 0..MEMORY_ADDRESS_TO_ID_SPLIT {
        // We compute the new address here and not at the end of the loop body
        // to avoid computing an unused address at the last iteration.
        if i != 0 {
            address = eval!(context, (address) + (component_data.n_instances()));
        }

        let id = input[i * 2];
        let multiplicity = input[i * 2 + 1];

        let tuple: &[Var] =
            &[context.constant(MEMORY_ADDRESS_TO_ID_RELATION_ID.into()), address, id];
        let numerator = eval!(context, -(multiplicity));
        acc.add_to_relation(context, numerator, tuple);
    }
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "memory_address_to_id".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        // This component does only yields
        &[]
    }

    fn log_size(
        &self,
        _preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
    ) -> Option<u32> {
        None
    }
}
