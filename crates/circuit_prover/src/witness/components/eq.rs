use crate::circuit_air::components::eq::N_TRACE_COLUMNS;
use crate::witness::components::prelude::*;

pub type InputType = [M31; 4];
pub type PackedInputType = [PackedM31; 4];

/// Retrieves the component's inputs from the context values, using the addresses provided in the
/// preprocessed trace.
#[allow(clippy::uninit_vec)]
pub fn extract_component_inputs(
    in0_address: &[usize],
    in1_address: &[usize],
    context_values: &[QM31],
) -> Vec<InputType> {
    let n_rows = in0_address.len();
    assert_eq!(n_rows, in1_address.len());

    let mut inputs = Vec::with_capacity(n_rows);
    unsafe {
        inputs.set_len(n_rows);
    }

    // Sanity check that the inputs are equal
    #[cfg(debug_assertions)]
    (in0_address.par_iter(), in1_address.par_iter()).into_par_iter().for_each(
        |(in0_address, in1_address)| {
            assert_eq!(
                context_values[*in0_address], context_values[*in1_address],
                "Eq gate: in0 and in1 must have equal values"
            );
        },
    );

    (inputs.par_iter_mut(), in0_address.par_iter()).into_par_iter().for_each(
        |(input, in0_address)| {
            *input = context_values[*in0_address].to_m31_array();
        },
    );

    inputs
}

pub fn write_trace(
    context_values: &[QM31],
    preprocessed_trace: &PreProcessedTrace,
) -> (ComponentTrace<N_TRACE_COLUMNS>, u32, LookupData) {
    let in0_address =
        preprocessed_trace.get_column(&PreProcessedColumnId { id: "eq_in0_address".to_owned() });
    let in1_address =
        preprocessed_trace.get_column(&PreProcessedColumnId { id: "eq_in1_address".to_owned() });

    let inputs = extract_component_inputs(in0_address, in1_address, context_values);

    let n_rows = inputs.len();
    assert_ne!(n_rows, 0);
    assert!(n_rows >= N_LANES);
    assert!(n_rows.is_power_of_two());
    let log_size = n_rows.ilog2();

    let packed_inputs = pack_values(&inputs);

    let preprocessed_columns = [in0_address, in1_address]
        .into_iter()
        .map(|col| Col::<SimdBackend, M31>::from_iter(col.iter().map(|&x| M31::from(x))).data)
        .collect_vec();

    let (trace, lookup_data) = write_trace_simd(packed_inputs, preprocessed_columns);

    (trace, log_size, lookup_data)
}

fn write_trace_simd(
    inputs: Vec<PackedInputType>,
    preprocessed_columns: Vec<Vec<PackedM31>>,
) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData) {
    let m31_gate_relation_id = PackedM31::broadcast(M31::from(378353459));
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;
    let (mut trace, mut lookup_data) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
        )
    };

    let [in0_address, in1_address] = preprocessed_columns.try_into().unwrap();

    (trace.par_iter_mut(), lookup_data.par_iter_mut(), inputs.into_par_iter())
        .into_par_iter()
        .enumerate()
        .for_each(|(row_index, (row, lookup_data, input))| {
            let in0_address = in0_address[row_index];
            let in1_address = in1_address[row_index];

            let in_col0 = input[0];
            *row[0] = in_col0;
            let in_col1 = input[1];
            *row[1] = in_col1;
            let in_col2 = input[2];
            *row[2] = in_col2;
            let in_col3 = input[3];
            *row[3] = in_col3;
            *lookup_data.in_0 =
                [m31_gate_relation_id, in0_address, in_col0, in_col1, in_col2, in_col3];
            *lookup_data.in_1 =
                [m31_gate_relation_id, in1_address, in_col0, in_col1, in_col2, in_col3];
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
pub struct LookupData {
    in_0: Vec<[PackedM31; 6]>,
    in_1: Vec<[PackedM31; 6]>,
}

pub fn write_interaction_trace(
    log_size: u32,
    lookup_data: LookupData,
    common_lookup_elements: &relations::CommonLookupElements,
) -> (Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>, SecureField) {
    let mut logup_gen = unsafe { LogupTraceGenerator::uninitialized(log_size) };

    // Sum logup terms in pairs.
    let mut col_gen = logup_gen.new_col();
    (col_gen.par_iter_mut(), &lookup_data.in_0, &lookup_data.in_1).into_par_iter().for_each(
        |(writer, values0, values1)| {
            let denom0: PackedQM31 = common_lookup_elements.combine(values0);
            let denom1: PackedQM31 = common_lookup_elements.combine(values1);
            writer.write_frac(denom0 + denom1, denom0 * denom1);
        },
    );
    col_gen.finalize_col();

    let (trace, claimed_sum) = logup_gen.finalize_last();

    (trace, claimed_sum)
}
