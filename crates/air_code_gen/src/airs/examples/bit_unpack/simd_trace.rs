use air_infra::core::prover_types::*;
use stwo_prover::core::backend::simd::m31::PackedBaseField;

use crate::code_gen::packed_types::*;

#[allow(non_snake_case)]
#[allow(clippy::useless_conversion)]
pub fn write_trace_row(
    dst: &mut [Vec<PackedBaseField>],
    BitUnpack__12_input: PackedUInt16,
    row_index: usize,
) {
    let col0 = BitUnpack__12_input.as_felt();
    dst[0][row_index] = col0;
    let deduction_tmp_2 =
        (BitUnpack__12_input) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col1 = deduction_tmp_2.as_felt();
    dst[1][row_index] = col1;
    let deduction_tmp_4 = (deduction_tmp_2) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col2 = deduction_tmp_4.as_felt();
    dst[2][row_index] = col2;
    let deduction_tmp_6 = (deduction_tmp_4) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col3 = deduction_tmp_6.as_felt();
    dst[3][row_index] = col3;
    let deduction_tmp_8 = (deduction_tmp_6) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col4 = deduction_tmp_8.as_felt();
    dst[4][row_index] = col4;
    let deduction_tmp_10 = (deduction_tmp_8) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col5 = deduction_tmp_10.as_felt();
    dst[5][row_index] = col5;
    let deduction_tmp_12 = (deduction_tmp_10) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col6 = deduction_tmp_12.as_felt();
    dst[6][row_index] = col6;
    let deduction_tmp_14 = (deduction_tmp_12) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col7 = deduction_tmp_14.as_felt();
    dst[7][row_index] = col7;
    let deduction_tmp_16 = (deduction_tmp_14) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col8 = deduction_tmp_16.as_felt();
    dst[8][row_index] = col8;
    let deduction_tmp_18 = (deduction_tmp_16) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col9 = deduction_tmp_18.as_felt();
    dst[9][row_index] = col9;
    let deduction_tmp_20 = (deduction_tmp_18) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col10 = deduction_tmp_20.as_felt();
    dst[10][row_index] = col10;
    let deduction_tmp_22 = (deduction_tmp_20) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col11 = deduction_tmp_22.as_felt();
    dst[11][row_index] = col11;
    let deduction_tmp_24 = (deduction_tmp_22) >> (PackedUInt16::broadcast(UInt16::from(1).into()));
    let col12 = deduction_tmp_24.as_felt();
    dst[12][row_index] = col12;
}
