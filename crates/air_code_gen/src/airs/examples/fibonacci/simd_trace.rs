use std::iter::zip;

use air_infra::core::prover_types::*;
use itertools::Itertools;
use num_traits::Zero;
use stwo_prover::core::air::Component;
use stwo_prover::core::backend::simd::column::BaseFieldVec;
use stwo_prover::core::backend::simd::m31::PackedBaseField;
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::poly::circle::{CanonicCoset, CircleEvaluation};
use stwo_prover::core::poly::BitReversedOrder;

use super::component::Fib__100;
use crate::code_gen::packed_types::*;

pub fn write_trace_simd(
    component: &Fib__100,
    secrets: &[PackedFelt],
) -> Vec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>> {
    let n_columns = component.trace_log_degree_bounds()[0].len();
    let mut trace_values = vec![vec![PackedBaseField::zero(); secrets.len()]; n_columns];
    for (i, secret) in secrets.iter().copied().enumerate() {
        super::simd_trace::write_trace_row(&mut trace_values, secret, i);
    }
    let trace_domains = trace_values
        .iter()
        .map(|col| {
            CanonicCoset::new(
                (col.len() * N_LANES)
                    .checked_ilog2()
                    .expect("Input not a power of 2!"),
            )
            .circle_domain()
        })
        .collect_vec();
    zip(trace_values, trace_domains)
        .map(|(eval, trace_domain)| {
            let length = eval.len() * N_LANES;
            let eval = BaseFieldVec { data: eval, length };
            CircleEvaluation::<SimdBackend, BaseField, BitReversedOrder>::new(trace_domain, eval)
        })
        .collect_vec()
}

#[allow(non_snake_case)]
#[allow(clippy::useless_conversion)]
pub fn write_trace_row(
    dst: &mut [Vec<PackedBaseField>],
    Fib__100_input: PackedFelt,
    row_index: usize,
) {
    let col0 = Fib__100_input;
    dst[0][row_index] = col0;
    let col1 = ((PackedFelt::broadcast(Felt::from(1).into()))
        * (PackedFelt::broadcast(Felt::from(1).into())))
        + ((col0) * (col0));
    dst[1][row_index] = col1;
    let col2 = ((col0) * (col0)) + ((col1) * (col1));
    dst[2][row_index] = col2;
    let col3 = ((col1) * (col1)) + ((col2) * (col2));
    dst[3][row_index] = col3;
    let col4 = ((col2) * (col2)) + ((col3) * (col3));
    dst[4][row_index] = col4;
    let col5 = ((col3) * (col3)) + ((col4) * (col4));
    dst[5][row_index] = col5;
    let col6 = ((col4) * (col4)) + ((col5) * (col5));
    dst[6][row_index] = col6;
    let col7 = ((col5) * (col5)) + ((col6) * (col6));
    dst[7][row_index] = col7;
    let col8 = ((col6) * (col6)) + ((col7) * (col7));
    dst[8][row_index] = col8;
    let col9 = ((col7) * (col7)) + ((col8) * (col8));
    dst[9][row_index] = col9;
    let col10 = ((col8) * (col8)) + ((col9) * (col9));
    dst[10][row_index] = col10;
    let col11 = ((col9) * (col9)) + ((col10) * (col10));
    dst[11][row_index] = col11;
    let col12 = ((col10) * (col10)) + ((col11) * (col11));
    dst[12][row_index] = col12;
    let col13 = ((col11) * (col11)) + ((col12) * (col12));
    dst[13][row_index] = col13;
    let col14 = ((col12) * (col12)) + ((col13) * (col13));
    dst[14][row_index] = col14;
    let col15 = ((col13) * (col13)) + ((col14) * (col14));
    dst[15][row_index] = col15;
    let col16 = ((col14) * (col14)) + ((col15) * (col15));
    dst[16][row_index] = col16;
    let col17 = ((col15) * (col15)) + ((col16) * (col16));
    dst[17][row_index] = col17;
    let col18 = ((col16) * (col16)) + ((col17) * (col17));
    dst[18][row_index] = col18;
    let col19 = ((col17) * (col17)) + ((col18) * (col18));
    dst[19][row_index] = col19;
    let col20 = ((col18) * (col18)) + ((col19) * (col19));
    dst[20][row_index] = col20;
    let col21 = ((col19) * (col19)) + ((col20) * (col20));
    dst[21][row_index] = col21;
    let col22 = ((col20) * (col20)) + ((col21) * (col21));
    dst[22][row_index] = col22;
    let col23 = ((col21) * (col21)) + ((col22) * (col22));
    dst[23][row_index] = col23;
    let col24 = ((col22) * (col22)) + ((col23) * (col23));
    dst[24][row_index] = col24;
    let col25 = ((col23) * (col23)) + ((col24) * (col24));
    dst[25][row_index] = col25;
    let col26 = ((col24) * (col24)) + ((col25) * (col25));
    dst[26][row_index] = col26;
    let col27 = ((col25) * (col25)) + ((col26) * (col26));
    dst[27][row_index] = col27;
    let col28 = ((col26) * (col26)) + ((col27) * (col27));
    dst[28][row_index] = col28;
    let col29 = ((col27) * (col27)) + ((col28) * (col28));
    dst[29][row_index] = col29;
    let col30 = ((col28) * (col28)) + ((col29) * (col29));
    dst[30][row_index] = col30;
    let col31 = ((col29) * (col29)) + ((col30) * (col30));
    dst[31][row_index] = col31;
    let col32 = ((col30) * (col30)) + ((col31) * (col31));
    dst[32][row_index] = col32;
    let col33 = ((col31) * (col31)) + ((col32) * (col32));
    dst[33][row_index] = col33;
    let col34 = ((col32) * (col32)) + ((col33) * (col33));
    dst[34][row_index] = col34;
    let col35 = ((col33) * (col33)) + ((col34) * (col34));
    dst[35][row_index] = col35;
    let col36 = ((col34) * (col34)) + ((col35) * (col35));
    dst[36][row_index] = col36;
    let col37 = ((col35) * (col35)) + ((col36) * (col36));
    dst[37][row_index] = col37;
    let col38 = ((col36) * (col36)) + ((col37) * (col37));
    dst[38][row_index] = col38;
    let col39 = ((col37) * (col37)) + ((col38) * (col38));
    dst[39][row_index] = col39;
    let col40 = ((col38) * (col38)) + ((col39) * (col39));
    dst[40][row_index] = col40;
    let col41 = ((col39) * (col39)) + ((col40) * (col40));
    dst[41][row_index] = col41;
    let col42 = ((col40) * (col40)) + ((col41) * (col41));
    dst[42][row_index] = col42;
    let col43 = ((col41) * (col41)) + ((col42) * (col42));
    dst[43][row_index] = col43;
    let col44 = ((col42) * (col42)) + ((col43) * (col43));
    dst[44][row_index] = col44;
    let col45 = ((col43) * (col43)) + ((col44) * (col44));
    dst[45][row_index] = col45;
    let col46 = ((col44) * (col44)) + ((col45) * (col45));
    dst[46][row_index] = col46;
    let col47 = ((col45) * (col45)) + ((col46) * (col46));
    dst[47][row_index] = col47;
    let col48 = ((col46) * (col46)) + ((col47) * (col47));
    dst[48][row_index] = col48;
    let col49 = ((col47) * (col47)) + ((col48) * (col48));
    dst[49][row_index] = col49;
    let col50 = ((col48) * (col48)) + ((col49) * (col49));
    dst[50][row_index] = col50;
    let col51 = ((col49) * (col49)) + ((col50) * (col50));
    dst[51][row_index] = col51;
    let col52 = ((col50) * (col50)) + ((col51) * (col51));
    dst[52][row_index] = col52;
    let col53 = ((col51) * (col51)) + ((col52) * (col52));
    dst[53][row_index] = col53;
    let col54 = ((col52) * (col52)) + ((col53) * (col53));
    dst[54][row_index] = col54;
    let col55 = ((col53) * (col53)) + ((col54) * (col54));
    dst[55][row_index] = col55;
    let col56 = ((col54) * (col54)) + ((col55) * (col55));
    dst[56][row_index] = col56;
    let col57 = ((col55) * (col55)) + ((col56) * (col56));
    dst[57][row_index] = col57;
    let col58 = ((col56) * (col56)) + ((col57) * (col57));
    dst[58][row_index] = col58;
    let col59 = ((col57) * (col57)) + ((col58) * (col58));
    dst[59][row_index] = col59;
    let col60 = ((col58) * (col58)) + ((col59) * (col59));
    dst[60][row_index] = col60;
    let col61 = ((col59) * (col59)) + ((col60) * (col60));
    dst[61][row_index] = col61;
    let col62 = ((col60) * (col60)) + ((col61) * (col61));
    dst[62][row_index] = col62;
    let col63 = ((col61) * (col61)) + ((col62) * (col62));
    dst[63][row_index] = col63;
    let col64 = ((col62) * (col62)) + ((col63) * (col63));
    dst[64][row_index] = col64;
    let col65 = ((col63) * (col63)) + ((col64) * (col64));
    dst[65][row_index] = col65;
    let col66 = ((col64) * (col64)) + ((col65) * (col65));
    dst[66][row_index] = col66;
    let col67 = ((col65) * (col65)) + ((col66) * (col66));
    dst[67][row_index] = col67;
    let col68 = ((col66) * (col66)) + ((col67) * (col67));
    dst[68][row_index] = col68;
    let col69 = ((col67) * (col67)) + ((col68) * (col68));
    dst[69][row_index] = col69;
    let col70 = ((col68) * (col68)) + ((col69) * (col69));
    dst[70][row_index] = col70;
    let col71 = ((col69) * (col69)) + ((col70) * (col70));
    dst[71][row_index] = col71;
    let col72 = ((col70) * (col70)) + ((col71) * (col71));
    dst[72][row_index] = col72;
    let col73 = ((col71) * (col71)) + ((col72) * (col72));
    dst[73][row_index] = col73;
    let col74 = ((col72) * (col72)) + ((col73) * (col73));
    dst[74][row_index] = col74;
    let col75 = ((col73) * (col73)) + ((col74) * (col74));
    dst[75][row_index] = col75;
    let col76 = ((col74) * (col74)) + ((col75) * (col75));
    dst[76][row_index] = col76;
    let col77 = ((col75) * (col75)) + ((col76) * (col76));
    dst[77][row_index] = col77;
    let col78 = ((col76) * (col76)) + ((col77) * (col77));
    dst[78][row_index] = col78;
    let col79 = ((col77) * (col77)) + ((col78) * (col78));
    dst[79][row_index] = col79;
    let col80 = ((col78) * (col78)) + ((col79) * (col79));
    dst[80][row_index] = col80;
    let col81 = ((col79) * (col79)) + ((col80) * (col80));
    dst[81][row_index] = col81;
    let col82 = ((col80) * (col80)) + ((col81) * (col81));
    dst[82][row_index] = col82;
    let col83 = ((col81) * (col81)) + ((col82) * (col82));
    dst[83][row_index] = col83;
    let col84 = ((col82) * (col82)) + ((col83) * (col83));
    dst[84][row_index] = col84;
    let col85 = ((col83) * (col83)) + ((col84) * (col84));
    dst[85][row_index] = col85;
    let col86 = ((col84) * (col84)) + ((col85) * (col85));
    dst[86][row_index] = col86;
    let col87 = ((col85) * (col85)) + ((col86) * (col86));
    dst[87][row_index] = col87;
    let col88 = ((col86) * (col86)) + ((col87) * (col87));
    dst[88][row_index] = col88;
    let col89 = ((col87) * (col87)) + ((col88) * (col88));
    dst[89][row_index] = col89;
    let col90 = ((col88) * (col88)) + ((col89) * (col89));
    dst[90][row_index] = col90;
    let col91 = ((col89) * (col89)) + ((col90) * (col90));
    dst[91][row_index] = col91;
    let col92 = ((col90) * (col90)) + ((col91) * (col91));
    dst[92][row_index] = col92;
    let col93 = ((col91) * (col91)) + ((col92) * (col92));
    dst[93][row_index] = col93;
    let col94 = ((col92) * (col92)) + ((col93) * (col93));
    dst[94][row_index] = col94;
    let col95 = ((col93) * (col93)) + ((col94) * (col94));
    dst[95][row_index] = col95;
    let col96 = ((col94) * (col94)) + ((col95) * (col95));
    dst[96][row_index] = col96;
    let col97 = ((col95) * (col95)) + ((col96) * (col96));
    dst[97][row_index] = col97;
    let col98 = ((col96) * (col96)) + ((col97) * (col97));
    dst[98][row_index] = col98;
}
