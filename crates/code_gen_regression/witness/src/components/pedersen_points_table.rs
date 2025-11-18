// This file was created by the AIR team.

#![allow(unused_parens)]
use cairo_air::components::pedersen_points_table::{
    Claim, InteractionClaim, LOG_SIZE, N_TRACE_COLUMNS,
};

use crate::witness::prelude::*;

pub type InputType = [M31; 1];
pub type PackedInputType = [PackedM31; 1];

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
    pub fn new() -> Self {
        Self::default()
    }

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
            packed_input.unpack().into_par_iter().for_each(|input| {
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

    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();
    todo!();

    (trace.par_iter_mut(), lookup_data.par_iter_mut())
        .into_par_iter()
        .enumerate()
        .for_each(|(row_index, (row, lookup_data))| {
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            todo!();
            *lookup_data.pedersen_points_table_0 = [
                seq_23,
                pedersen_points_0,
                pedersen_points_1,
                pedersen_points_2,
                pedersen_points_3,
                pedersen_points_4,
                pedersen_points_5,
                pedersen_points_6,
                pedersen_points_7,
                pedersen_points_8,
                pedersen_points_9,
                pedersen_points_10,
                pedersen_points_11,
                pedersen_points_12,
                pedersen_points_13,
                pedersen_points_14,
                pedersen_points_15,
                pedersen_points_16,
                pedersen_points_17,
                pedersen_points_18,
                pedersen_points_19,
                pedersen_points_20,
                pedersen_points_21,
                pedersen_points_22,
                pedersen_points_23,
                pedersen_points_24,
                pedersen_points_25,
                pedersen_points_26,
                pedersen_points_27,
                pedersen_points_28,
                pedersen_points_29,
                pedersen_points_30,
                pedersen_points_31,
                pedersen_points_32,
                pedersen_points_33,
                pedersen_points_34,
                pedersen_points_35,
                pedersen_points_36,
                pedersen_points_37,
                pedersen_points_38,
                pedersen_points_39,
                pedersen_points_40,
                pedersen_points_41,
                pedersen_points_42,
                pedersen_points_43,
                pedersen_points_44,
                pedersen_points_45,
                pedersen_points_46,
                pedersen_points_47,
                pedersen_points_48,
                pedersen_points_49,
                pedersen_points_50,
                pedersen_points_51,
                pedersen_points_52,
                pedersen_points_53,
                pedersen_points_54,
                pedersen_points_55,
            ];
            let mult_at_row = *mults.get(row_index).unwrap_or(&PackedM31::zero());
            *row[0] = mult_at_row;
            *lookup_data.mults = mult_at_row;
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    pedersen_points_table_0: Vec<[PackedM31; 57]>,
    mults: Vec<PackedM31>,
}

pub struct InteractionClaimGenerator {
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        pedersen_points_table: &relations::PedersenPointsTable,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(LOG_SIZE);

        // Sum last logup term.
        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.pedersen_points_table_0,
            self.lookup_data.mults,
        )
            .into_par_iter()
            .for_each(|(writer, values, mults)| {
                let denom = pedersen_points_table.combine(values);
                writer.write_frac(-PackedQM31::one() * mults, denom);
            });
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
