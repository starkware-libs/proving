#![allow(unused_parens)]
#![allow(dead_code)]
use cairo_air::components::range_check_18::{Claim, InteractionClaim, N_TRACE_COLUMNS};

use crate::witness::prelude::*;

pub type InputType = [M31; 1];
pub type PackedInputType = [PackedM31; 1];

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn new(inputs: Vec<InputType>) -> Self {
        Self { inputs }
    }

    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> (Claim, InteractionClaimGenerator) {
        let n_rows = self.inputs.len();
        assert_ne!(n_rows, 0);
        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();
        self.inputs.resize(size, *self.inputs.first().unwrap());
        let packed_inputs = pack_values(&self.inputs);

        let (trace, lookup_data) = write_trace_simd(n_rows, packed_inputs);
        tree_builder.extend_evals(trace.to_evals());

        (
            Claim { log_size },
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        )
    }

    pub fn add_packed_input(&self, input: &PackedInputType) {
        unimplemented!("Implement manually");
    }

    pub fn add_packed_inputs(&self, inputs: &[PackedInputType]) {
        unimplemented!("Implement manually");
    }
}

fn write_trace_simd(
    n_rows: usize,
    inputs: Vec<PackedInputType>,
) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData) {
    unimplemented!()
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    range_check_18_0: Vec<[PackedM31; 1]>,
}

pub struct InteractionClaimGenerator {
    log_size: u32,
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        range_check_18: &relations::RangeCheck_18,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        // Sum last logup term.
        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.range_check_18_0)
            .into_par_iter()
            .for_each(|(writer, values)| {
                let denom = range_check_18.combine(values);
                writer.write_frac(-PackedQM31::one(), denom);
            });
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
