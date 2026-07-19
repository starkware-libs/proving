#[cfg(test)]
use std::array::from_fn;

#[cfg(test)]
use air_infra::const_felt252_width27;
use air_infra::core::air_fn::AirBuilder;
use air_infra::core::expressions::felt252width27_expr::Felt252Width27Expr;
use air_infra::core::variables::ExtTable;
use stwo_cairo_common::preprocessed_columns::poseidon::{N_WORDS, PoseidonRoundKeys};
#[cfg(test)]
use stwo_cairo_common::preprocessed_columns::poseidon_round_keys::round_keys;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;

/// A constant table for the round keys of poseidon.
/// Accessed by the PoseidonRoundKeys component through an external column call.
/// There are 35 rows, where `i` contains 3 keys (Felt252) in Width27 form.
/// The first 4 and last 4 rows represent triplets of keys corresponding to full rounds 1..5 and
/// 88..92 (in particular, the very last key is 0s, as it is a "fake" add key round).
/// Rows 4..31 correspond to "keys" belonging to three consecutive partial rounds, that are added
/// only to the z variables, and dividied by 2.
#[derive(Debug, Clone, Default)]
pub struct Keys {}
impl ExtTable for Keys {
    type T = [Felt252Width27Expr; 3];

    fn call_impl(&self, _air_builder: &mut AirBuilder) -> Self::T {
        #[cfg(test)]
        if _air_builder.is_run_mode() {
            let row_number = _air_builder.row_number().expect("Row number not set");
            return from_fn(|i| const_felt252_width27!(round_keys(row_number)[i]));
        }

        Self::T::default()
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        (0..N_WORDS)
            .map(|i| Box::new(PoseidonRoundKeys::new(i)) as Box<dyn PreProcessedColumn>)
            .collect()
    }
}
