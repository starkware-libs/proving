use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::fields::FieldExpOps;

use super::FibInput;

pub fn write_trace_row(dst: &mut [Vec<BaseField>], input: FibInput, row_index: usize) {
    let col0 = input.a;
    let col1 = input.b;
    dst[0][row_index] = col0.into();
    dst[1][row_index] = col1.into();
    for i in 2..dst.len() {
        dst[i][row_index] = dst[i - 1][row_index].square() + dst[i - 2][row_index].square();
    }
}
