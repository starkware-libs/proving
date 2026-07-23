use crate::components::prelude::*;

const EXPAND_BITS: usize = 2;
const LOW_BITS: usize = 10;
pub const N_TRACE_COLUMNS: usize = 1 << (EXPAND_BITS * 2);
pub const N_INTERACTION_COLUMNS: usize = 2 << (EXPAND_BITS * 2);
pub const VERIFY_BITWISE_XOR_12_RELATION_ID: u32 = 648362599;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let a_low =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "bitwise_xor_10_0".into() });
    let b_low =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "bitwise_xor_10_1".into() });
    let c_low =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "bitwise_xor_10_2".into() });

    let mut multiplicities = input.iter();
    let relation_id_const = context.constant(VERIFY_BITWISE_XOR_12_RELATION_ID.into());
    for i in 0..1 << EXPAND_BITS {
        for j in 0..1 << EXPAND_BITS {
            let a = eval!(context, (a_low) + (context.constant((i << LOW_BITS).into())));
            let b = eval!(context, (b_low) + (context.constant((j << LOW_BITS).into())));
            let c = eval!(context, (c_low) + (context.constant(((i ^ j) << LOW_BITS).into())));

            let multiplicity = multiplicities.next().unwrap();
            let numerator = eval!(context, -(*multiplicity));
            let tuple = [relation_id_const, a, b, c];
            acc.add_to_relation(context, numerator, &tuple);
        }
    }
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "verify_bitwise_xor_12".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
        // Verify this component has 2 ** 20 rows
        let size_bit = component_data.get_n_instances_bit(context, 20);
        eq(context, size_bit, context.one());
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &RELATION_USES_PER_ROW
    }

    fn log_size(
        &self,
        _preprocessed_column_log_sizes: &OrderedHashMap<PreProcessedColumnId, u32>,
    ) -> Option<u32> {
        Some(20)
    }
}
