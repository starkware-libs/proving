use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use itertools::Itertools;
use stwo::core::fields::qm31::QM31;

use crate::constraint_eval::ComponentDataTrait;
use crate::proof::InteractionAtOods;

pub struct TestComponentData {
    trace: Vec<Var>,
    interaction_trace: Vec<InteractionAtOods<Var>>,
    n_instances_var: Var,
    n_instances_bits: Vec<Var>,
}

impl TestComponentData {
    pub fn from_values(
        context: &mut Context<QM31>,
        trace_values: &[QM31],
        interaction_values: &[QM31],
        last_row_sum: QM31,
        n_instances: usize,
    ) -> Self {
        let trace = trace_values.iter().map(|v| context.new_var(*v)).collect_vec();
        let mut interaction_trace = interaction_values
            .iter()
            .flat_map(|v| v.to_m31_array())
            .map(|m31| InteractionAtOods { at_oods: context.new_var(m31.into()), at_prev: None })
            .collect_vec();
        if !interaction_trace.is_empty() {
            let last_row_sum_m31s = last_row_sum.to_m31_array();
            let interaction_trace_len = interaction_trace.len();
            for i in 0..4 {
                interaction_trace[interaction_trace_len - 4 + i].at_prev =
                    Some(context.new_var(last_row_sum_m31s[i].into()));
            }
        }
        let n_instances_bits = (0..31)
            .map(|bit_pos| {
                let bit = (n_instances >> bit_pos) & 1;
                context.new_var(bit.into())
            })
            .collect_vec();
        Self {
            trace,
            interaction_trace,
            n_instances_var: context.new_var(n_instances.into()),
            n_instances_bits,
        }
    }
}

impl<Value: IValue> ComponentDataTrait<Value> for TestComponentData {
    fn trace_columns(&self) -> &[Var] {
        &self.trace
    }

    fn interaction_columns(&self) -> &[InteractionAtOods<Var>] {
        &self.interaction_trace
    }

    fn n_instances(&self) -> Var {
        self.n_instances_var
    }

    fn get_n_instances_bit(&self, _context: &mut Context<Value>, bit: usize) -> Var {
        self.n_instances_bits[bit]
    }

    fn max_component_size_bits(&self) -> usize {
        self.n_instances_bits.len()
    }
}
