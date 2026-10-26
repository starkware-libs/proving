use std::collections::HashMap;

use circuits::context::*;
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::eq;
use circuits_stark_verifier::constraint_eval::*;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

// Create a variable with the evaluation of seq_k where k is the log-height of
// the component. The height is taken from component_data.get_n_instances_bits.
pub fn seq_of_component_size<Value: IValue>(
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    preprocessed_columns: &HashMap<PreProcessedColumnId, Var>,
) -> Var {
    // Compute:
    //      sum_bits = size_bits[0] + size_bits[1] + ... + size_bits[MAX_BITS]
    //      result = sum_k size_bits[k] * seq_k  (for all seq_k columns in the preprocessed trace)
    let mut sum_bits = context.zero();
    let mut result = context.zero();

    for log_size in 0..component_data.max_component_size_bits() {
        let seq_name = PreProcessedColumnId { id: format!("seq_{log_size}") };
        let Some(seq_value) = preprocessed_columns.get(&seq_name) else {
            // Our preprocessed trace doesn't contain a seq column of this size
            continue;
        };

        let bit = component_data.get_n_instances_bit(context, log_size);

        sum_bits = eval!(context, (sum_bits) + (bit));
        result = eval!(context, (result) + ((bit) * (*seq_value)))
    }

    // Assert that the component size was one of the supported sizes
    eq(context, sum_bits, context.one());

    result
}
