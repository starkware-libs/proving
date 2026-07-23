use crate::circuit_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 4;
pub const N_INTERACTION_COLUMNS: usize = 4;

pub struct Eval {
    pub log_size: u32,
    pub common_lookup_elements: relations::CommonLookupElements,
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        self.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let m31_gate_relation_id = E::F::from(M31::from(378353459));
        let in0_address =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "eq_in0_address".to_owned() });
        let in1_address =
            eval.get_preprocessed_column(PreProcessedColumnId { id: "eq_in1_address".to_owned() });

        let in_col0 = eval.next_trace_mask();
        let in_col1 = eval.next_trace_mask();
        let in_col2 = eval.next_trace_mask();
        let in_col3 = eval.next_trace_mask();

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                m31_gate_relation_id.clone(),
                in0_address.clone(),
                in_col0.clone(),
                in_col1.clone(),
                in_col2.clone(),
                in_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                m31_gate_relation_id.clone(),
                in1_address.clone(),
                in_col0.clone(),
                in_col1.clone(),
                in_col2.clone(),
                in_col3.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}
