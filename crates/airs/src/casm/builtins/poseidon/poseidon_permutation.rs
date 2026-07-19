use air_infra::const_felt252_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::felt252width27_expr::Felt252Width27Expr;
use serde::Serialize;

use super::cube252::*;
use super::full_round::*;
use super::linear_combination::*;
use super::partial_round::*;
use crate::felt252_utils::felt252_packing27::*;

const INITIAL_ROUND_KEYS: [[u128; 2]; 3] = [
    [305131386282307568993834265693067411055, 8671609373003262111106592235269533881],
    [81935731621622936992162859583385438404, 4665086171194536676173694368533725786],
    [59381497358581182462289361086406361865, 4837059104951847278822944255558666187],
];
const FULL_TO_PARTIAL_KEYS: [[u128; 2]; 2] = [
    [138409473871519061722290633027883440116, 10339346405861548376029761706113082800],
    [27222899939055784260963005711467436065, 6423702975204375043858926045261996345],
];
const PARTIAL_TO_FULL_KEYS: [[u128; 2]; 2] = [
    [194850633509866445394007900226536949759, 4119386164429252691021348544948421076],
    [108360149007852558976572667518806803677, 836575275863554585564957135709925185],
];
/// Computes and verifies a full Poseidon (Hades) permutation.
/// Note that both input and output packed felts are not range checked in the air, as it is expected
/// that the caller has them in unpacked or otherwise range-checked forms. It is crucial for
/// soundness that they are indeed range checked.
#[derive(Clone, Debug, Serialize)]
pub struct PoseidonHadesPermutation {}

impl AirFn for PoseidonHadesPermutation {
    type ExtIn = ();
    type In = [Felt252Width27Expr; 3];
    type Out = [Felt252Width27Expr; 3];

    fn call(&self, air_builder: &mut AirBuilder, _: (), state: Self::In) -> Self::Out {
        // Initial add round.
        let keys = INITIAL_ROUND_KEYS.map(|[x, y]| const_felt252_expr!(x, y).into());
        let state = state
            .into_iter()
            .zip(keys)
            .map(|(x, k)| air_builder.call(&LinearCombination::new([1, 1]), [x, k]))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        // First four full rounds.
        let [x4, y4, z4] = air_builder.chain_lookup_call(&PoseidonFullRoundChain {}, state, 0, 4);
        // x4 and y4 are not cubed but do enter linear combinations, so must be range checked.
        air_builder.lookup_call(&RangeCheck252Width27 {}, (), x4.clone());
        air_builder.lookup_call(&RangeCheck252Width27 {}, (), y4.clone());
        // Transition from full round state to partial round state (manually computing the first two
        // partial rounds' z-s).
        let [key_z5, key_z6] = FULL_TO_PARTIAL_KEYS.map(|[x, y]| const_felt252_expr!(x, y).into());
        let z4_3 = air_builder.lookup_call(&Cube252 {}, (), z4.clone());
        let z5 = air_builder
            .call(&LinearCombination::new([1, 1, -2, 1]), [x4.clone(), y4, z4_3.clone(), key_z5]);
        let z5_3 = air_builder.lookup_call(&Cube252 {}, (), z5.clone());
        let z6 = air_builder
            .call(&LinearCombination::new([4, 2, -2, 1]), [x4, z4_3.clone(), z5_3.clone(), key_z6]);
        // The remaining 81 partial rounds (in a 27x3 chain).
        let [z85_3, z86, z86_3, z87] = air_builder.chain_lookup_call(
            &Poseidon3PartialRoundsChain {},
            [z4_3, z5, z5_3, z6],
            4,
            27,
        );
        // Transition from final partial rounds state to full round state.
        let [key_y87, key_x87] =
            PARTIAL_TO_FULL_KEYS.map(|[x, y]| const_felt252_expr!(x, y).into());
        let y87 = air_builder
            .call(&LinearCombination::new([4, 2, 1, 1]), [z85_3, z86, z86_3.clone(), key_y87]);
        let x87 = air_builder.call(
            &LinearCombination::new([4, 2, 1, 1]),
            [z86_3, z87.clone(), y87.clone(), key_x87],
        );

        // Final four full rounds.
        air_builder.chain_lookup_call(&PoseidonFullRoundChain {}, [x87, y87, z87], 31, 4)
    }
}
