#![allow(unused_parens)]
#![allow(unused_imports)]
use std::iter::zip;

use air_structs_derive::SubComponentInputs;
use itertools::{chain, zip_eq, Itertools};
use num_traits::{One, Zero};
use prover_types::cpu::*;
use prover_types::simd::*;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use stwo_air_utils::trace::component_trace::ComponentTrace;
use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
use stwo_prover::constraint_framework::logup::LogupTraceGenerator;
use stwo_prover::constraint_framework::Relation;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::simd::column::BaseColumn;
use stwo_prover::core::backend::simd::conversion::Unpack;
use stwo_prover::core::backend::simd::m31::{PackedM31, LOG_N_LANES, N_LANES};
use stwo_prover::core::backend::simd::qm31::PackedQM31;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::{BackendForChannel, Col, Column};
use stwo_prover::core::channel::{Channel, MerkleChannel};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::FieldExpOps;
use stwo_prover::core::pcs::TreeBuilder;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::core::utils::bit_reverse_coset_to_circle_domain_order;

use super::component::{Claim, InteractionClaim};
use crate::components::pack_values;
use crate::relations;

pub type InputType = [M31; 2];
pub type PackedInputType = [PackedM31; 2];
const N_TRACE_COLUMNS: usize = 22;

#[derive(Default)]
pub struct ClaimGenerator {
    pub inputs: Vec<InputType>,
}
impl ClaimGenerator {
    pub fn new(inputs: Vec<InputType>) -> Self {
        Self { inputs }
    }

    pub fn write_trace<MC: MerkleChannel>(
        mut self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
    ) -> (Claim, InteractionClaimGenerator)
    where
        SimdBackend: BackendForChannel<MC>,
    {
        let n_rows = self.inputs.len();
        assert_ne!(n_rows, 0);
        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let need_padding = n_rows != size;

        if need_padding {
            self.inputs.resize(size, *self.inputs.first().unwrap());
            bit_reverse_coset_to_circle_domain_order(&mut self.inputs);
        }

        let packed_inputs = pack_values(&self.inputs);
        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        (
            Claim { n_rows },
            InteractionClaimGenerator {
                n_rows,
                lookup_data,
            },
        )
    }

    pub fn add_input(&self, input: &InputType) {
        todo!()
    }

    pub fn add_inputs(&self, _inputs: &[InputType]) {
        todo!()
    }
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
fn write_trace_simd(inputs: Vec<PackedInputType>) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData) {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;
    let (mut trace, mut lookup_data) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
        )
    };

    trace
        .par_iter_mut()
        .zip(inputs.par_iter())
        .zip(lookup_data.par_iter_mut())
        .for_each(|((row, narrow_fib_num_steps_20_input), lookup_data)| {
            let input_tmp_1ddf3_0 = [
                narrow_fib_num_steps_20_input[0],
                narrow_fib_num_steps_20_input[1],
            ];
            let input_col0 = input_tmp_1ddf3_0[0];
            *row[0] = input_col0;
            let input_col1 = input_tmp_1ddf3_0[1];
            *row[1] = input_col1;

            // Fib Step.

            let col2 = (((input_col0) * (input_col0)) + ((input_col1) * (input_col1)));
            *row[2] = col2;

            // Fib Step.

            let col3 = (((input_col1) * (input_col1)) + ((col2) * (col2)));
            *row[3] = col3;

            // Fib Step.

            let col4 = (((col2) * (col2)) + ((col3) * (col3)));
            *row[4] = col4;

            // Fib Step.

            let col5 = (((col3) * (col3)) + ((col4) * (col4)));
            *row[5] = col5;

            // Fib Step.

            let col6 = (((col4) * (col4)) + ((col5) * (col5)));
            *row[6] = col6;

            // Fib Step.

            let col7 = (((col5) * (col5)) + ((col6) * (col6)));
            *row[7] = col7;

            // Fib Step.

            let col8 = (((col6) * (col6)) + ((col7) * (col7)));
            *row[8] = col8;

            // Fib Step.

            let col9 = (((col7) * (col7)) + ((col8) * (col8)));
            *row[9] = col9;

            // Fib Step.

            let col10 = (((col8) * (col8)) + ((col9) * (col9)));
            *row[10] = col10;

            // Fib Step.

            let col11 = (((col9) * (col9)) + ((col10) * (col10)));
            *row[11] = col11;

            // Fib Step.

            let col12 = (((col10) * (col10)) + ((col11) * (col11)));
            *row[12] = col12;

            // Fib Step.

            let col13 = (((col11) * (col11)) + ((col12) * (col12)));
            *row[13] = col13;

            // Fib Step.

            let col14 = (((col12) * (col12)) + ((col13) * (col13)));
            *row[14] = col14;

            // Fib Step.

            let col15 = (((col13) * (col13)) + ((col14) * (col14)));
            *row[15] = col15;

            // Fib Step.

            let col16 = (((col14) * (col14)) + ((col15) * (col15)));
            *row[16] = col16;

            // Fib Step.

            let col17 = (((col15) * (col15)) + ((col16) * (col16)));
            *row[17] = col17;

            // Fib Step.

            let col18 = (((col16) * (col16)) + ((col17) * (col17)));
            *row[18] = col18;

            // Fib Step.

            let col19 = (((col17) * (col17)) + ((col18) * (col18)));
            *row[19] = col19;

            // Fib Step.

            let col20 = (((col18) * (col18)) + ((col19) * (col19)));
            *row[20] = col20;

            // Fib Step.

            let col21 = (((col19) * (col19)) + ((col20) * (col20)));
            *row[21] = col21;

            *lookup_data.narrow_fib_num_steps_20_0 = [input_col0, input_col1, col20, col21];
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    narrow_fib_num_steps_20_0: Vec<[PackedM31; 4]>,
}

pub struct InteractionClaimGenerator {
    n_rows: usize,
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace<MC: MerkleChannel>(
        self,
        tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, MC>,
        narrow_fib_num_steps_20: &relations::NarrowFibNumSteps20,
    ) -> InteractionClaim
    where
        SimdBackend: BackendForChannel<MC>,
    {
        let log_size = std::cmp::max(self.n_rows.next_power_of_two().ilog2(), LOG_N_LANES);
        let mut logup_gen = LogupTraceGenerator::new(log_size);

        // Sum last logup term.
        let mut col_gen = logup_gen.new_col();
        for (i, values) in self
            .lookup_data
            .narrow_fib_num_steps_20_0
            .iter()
            .enumerate()
        {
            let denom = narrow_fib_num_steps_20.combine(values);
            col_gen.write_frac(i, -PackedQM31::one(), denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
