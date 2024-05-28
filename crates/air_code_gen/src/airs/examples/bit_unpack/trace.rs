use air_infra::core::prover_types::*;
use stwo_prover::core::fields::m31::BaseField;

#[allow(non_snake_case)]
pub fn write_trace_row(dst: &mut [Vec<BaseField>], BitUnpack__12_input: UInt16, row_index: usize) {
    let col0 = BitUnpack__12_input.as_felt();
    dst[0][row_index] = col0.into();
    let deduction_tmp_2 = (BitUnpack__12_input) >> (UInt16::from(1));
    let col1 = deduction_tmp_2.as_felt();
    dst[1][row_index] = col1.into();
    let deduction_tmp_4 = (deduction_tmp_2) >> (UInt16::from(1));
    let col2 = deduction_tmp_4.as_felt();
    dst[2][row_index] = col2.into();
    let deduction_tmp_6 = (deduction_tmp_4) >> (UInt16::from(1));
    let col3 = deduction_tmp_6.as_felt();
    dst[3][row_index] = col3.into();
    let deduction_tmp_8 = (deduction_tmp_6) >> (UInt16::from(1));
    let col4 = deduction_tmp_8.as_felt();
    dst[4][row_index] = col4.into();
    let deduction_tmp_10 = (deduction_tmp_8) >> (UInt16::from(1));
    let col5 = deduction_tmp_10.as_felt();
    dst[5][row_index] = col5.into();
    let deduction_tmp_12 = (deduction_tmp_10) >> (UInt16::from(1));
    let col6 = deduction_tmp_12.as_felt();
    dst[6][row_index] = col6.into();
    let deduction_tmp_14 = (deduction_tmp_12) >> (UInt16::from(1));
    let col7 = deduction_tmp_14.as_felt();
    dst[7][row_index] = col7.into();
    let deduction_tmp_16 = (deduction_tmp_14) >> (UInt16::from(1));
    let col8 = deduction_tmp_16.as_felt();
    dst[8][row_index] = col8.into();
    let deduction_tmp_18 = (deduction_tmp_16) >> (UInt16::from(1));
    let col9 = deduction_tmp_18.as_felt();
    dst[9][row_index] = col9.into();
    let deduction_tmp_20 = (deduction_tmp_18) >> (UInt16::from(1));
    let col10 = deduction_tmp_20.as_felt();
    dst[10][row_index] = col10.into();
    let deduction_tmp_22 = (deduction_tmp_20) >> (UInt16::from(1));
    let col11 = deduction_tmp_22.as_felt();
    dst[11][row_index] = col11.into();
    let deduction_tmp_24 = (deduction_tmp_22) >> (UInt16::from(1));
    let col12 = deduction_tmp_24.as_felt();
    dst[12][row_index] = col12.into();
}
