#![allow(unused)]
use std::array::from_fn;

use num_traits::{One, Zero};
use starknet_types_core::curve::ProjectivePoint;
use stwo::prover::backend::simd::conversion::{Pack, Unpack};
use stwo::prover::backend::simd::m31::PackedM31;
use stwo_cairo_common::preprocessed_columns::pedersen::{PEDERSEN_TABLE_9, PEDERSEN_TABLE_18};
use stwo_cairo_common::prover_types::cpu::{Felt252, M31};
use stwo_cairo_common::prover_types::simd::PackedFelt252;

pub struct PackedPedersenPointsTableWindowBits9 {}
impl PackedPedersenPointsTableWindowBits9 {
    pub fn deduce_output([input]: [PackedM31; 1]) -> [PackedFelt252; 2] {
        let arr = input.to_array().map(|i| PEDERSEN_TABLE_9.get_row_coordinates(i.0 as usize));
        <_ as Pack>::pack(arr)
    }
}

pub struct PackedPedersenPointsTableWindowBits18 {}
impl PackedPedersenPointsTableWindowBits18 {
    pub fn deduce_output([input]: [PackedM31; 1]) -> [PackedFelt252; 2] {
        let arr = input.to_array().map(|i| PEDERSEN_TABLE_18.get_row_coordinates(i.0 as usize));
        <_ as Pack>::pack(arr)
    }
}

#[derive(Debug)]
pub struct PartialEcMul<const NUM_WINDOWS: usize> {}
impl<const NUM_WINDOWS: usize> PartialEcMul<NUM_WINDOWS> {
    pub fn deduce_output(
        chain: M31,
        round: M31,
        (m_shifted, accumulator): ([M31; NUM_WINDOWS], [Felt252; 2]),
    ) -> (M31, M31, ([M31; NUM_WINDOWS], [Felt252; 2])) {
        assert_eq!(252 % NUM_WINDOWS, 0);
        let window_bits = 252 / NUM_WINDOWS;
        let rows_per_window = 1 << window_bits;

        let round_usize = round.0 as usize;
        assert!(round_usize < 2 * NUM_WINDOWS);

        let accumulator_point =
            ProjectivePoint::from_affine(accumulator[0].into(), accumulator[1].into())
                .expect("The accumulator should contain a curve point");

        let window_value = m_shifted[0].0 as usize;
        let table_row = round_usize * rows_per_window + window_value;
        let affine_point = match window_bits {
            18 => PEDERSEN_TABLE_18.get_row(table_row),
            9 => PEDERSEN_TABLE_9.get_row(table_row),
            _ => panic!("Unsupported window_bits value {window_bits}"),
        };
        let table_point = ProjectivePoint::from_affine(affine_point.x, affine_point.y)
            .expect("Table point should be on curve");

        let new_accumulator_point =
            (accumulator_point + table_point).to_affine().expect("New accumulator is not infinity");
        let new_accumulator_x: Felt252 = new_accumulator_point.x().into();
        let new_accumulator_y: Felt252 = new_accumulator_point.y().into();

        let new_m_shifted: [M31; NUM_WINDOWS] =
            from_fn(|i| if i < m_shifted.len() - 1 { m_shifted[i + 1] } else { M31::zero() });

        (chain, round + M31::one(), (new_m_shifted, [new_accumulator_x, new_accumulator_y]))
    }
}

pub struct PackedPartialEcMul<const NUM_WINDOWS: usize> {}
impl<const NUM_WINDOWS: usize> PackedPartialEcMul<NUM_WINDOWS> {
    pub fn deduce_output(
        input: (PackedM31, PackedM31, ([PackedM31; NUM_WINDOWS], [PackedFelt252; 2])),
    ) -> (PackedM31, PackedM31, ([PackedM31; NUM_WINDOWS], [PackedFelt252; 2])) {
        let unpacked_inputs = input.unpack();
        <_ as Pack>::pack(unpacked_inputs.map(|(chain, round, state)| {
            PartialEcMul::<NUM_WINDOWS>::deduce_output(chain, round, state)
        }))
    }
}

pub type PackedPartialEcMulWindowBits18 = PackedPartialEcMul<14>;
pub type PackedPartialEcMulWindowBits9 = PackedPartialEcMul<28>;
