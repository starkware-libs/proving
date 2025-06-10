#![allow(unused_parens)]
use cairo_air::components::range_check_3_6_6_3::{
    Claim, InteractionClaim, LOG_SIZE, N_TRACE_COLUMNS,
};

use crate::witness::prelude::*;

pub type InputType = [M31; 4];
pub type PackedInputType = [PackedM31; 4];

pub struct ClaimGenerator {
    pub mults: AtomicMultiplicityColumn,
}
impl Default for ClaimGenerator {
    fn default() -> Self {
        Self {
            mults: AtomicMultiplicityColumn::new(1 << LOG_SIZE),
        }
    }
}
impl ClaimGenerator {
    pub fn write_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> (Claim, InteractionClaimGenerator) {
        let mults = self.mults.into_simd_vec();

        let (trace, lookup_data) = write_trace_simd(mults);
        tree_builder.extend_evals(trace.to_evals());

        (Claim {}, InteractionClaimGenerator { lookup_data })
    }

    pub fn add_input(&self, _input: &InputType) {
        todo!()
    }

    pub fn add_packed_inputs(&self, packed_inputs: &[PackedInputType]) {
        packed_inputs.into_par_iter().for_each(|packed_input| {
            packed_input.unpack().into_iter().for_each(|input| {
                self.add_input(&input);
            });
        });
    }
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
fn write_trace_simd(mults: Vec<PackedM31>) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData) {
    let log_n_packed_rows = LOG_SIZE - LOG_N_LANES;
    let (mut trace, mut lookup_data) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(LOG_SIZE),
            LookupData::uninitialized(log_n_packed_rows),
        )
    };

    let rangecheck_3_6_6_3_0 = RangeCheck::new([3, 6, 6, 3], 0);
    let rangecheck_3_6_6_3_1 = RangeCheck::new([3, 6, 6, 3], 1);
    let rangecheck_3_6_6_3_2 = RangeCheck::new([3, 6, 6, 3], 2);
    let rangecheck_3_6_6_3_3 = RangeCheck::new([3, 6, 6, 3], 3);

    (trace.par_iter_mut(), lookup_data.par_iter_mut())
        .into_par_iter()
        .enumerate()
        .for_each(|(row_index, (mut row, lookup_data))| {
            let rangecheck_3_6_6_3_0 = rangecheck_3_6_6_3_0.packed_at(row_index);
            let rangecheck_3_6_6_3_1 = rangecheck_3_6_6_3_1.packed_at(row_index);
            let rangecheck_3_6_6_3_2 = rangecheck_3_6_6_3_2.packed_at(row_index);
            let rangecheck_3_6_6_3_3 = rangecheck_3_6_6_3_3.packed_at(row_index);
            *lookup_data.range_check_3_6_6_3_0 = [
                rangecheck_3_6_6_3_0,
                rangecheck_3_6_6_3_1,
                rangecheck_3_6_6_3_2,
                rangecheck_3_6_6_3_3,
            ];
            let mult_at_row = *mults.get(row_index).unwrap_or(&PackedM31::zero());
            *row[0] = mult_at_row;
            *lookup_data.mults = mult_at_row;
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    range_check_3_6_6_3_0: Vec<[PackedM31; 4]>,
    mults: Vec<PackedM31>,
}

pub struct InteractionClaimGenerator {
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        range_check_3_6_6_3: &relations::RangeCheck_3_6_6_3,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(LOG_SIZE);

        // Sum last logup term.
        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_3_6_6_3_0,
            self.lookup_data.mults,
        )
            .into_par_iter()
            .for_each(|(writer, values, mults)| {
                let denom = range_check_3_6_6_3.combine(values);
                writer.write_frac(-PackedQM31::one() * mults, denom);
            });
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
