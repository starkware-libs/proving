use std::array::from_fn;

use air_infra::casm_state::CasmAddress;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::variables::AirVar;
use air_infra::felt252_id_memory::memory::{CasmId, Felt252IdMemory};
use air_infra::felt252_id_memory::read_positive::ReadPositive;
use air_infra::range_check::range_check;
use serde::Serialize;

// The QM31ReadReduced inline AirFn.
// Reads a 144-bit value from the memory representing a packed QM31 element and outputs its M31
// coordinates and its id. Constrains the value to be in reduced form.
#[derive(Debug, Serialize, Default)]
pub struct QM31ReadReduced {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for QM31ReadReduced {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = ([FeltExpr; 4], CasmId);

    fn call(&self, ab: &mut AirBuilder, _: (), addr: Self::In) -> Self::Out {
        // Read the 144-bit value from the memory.
        let (value, id) = ab
            .call(&ReadPositive { num_bits: 4 * 4 * 9, memory: self.memory.clone() }, addr.clone());
        let limbs = value.as_felts();

        // Constrain the coordinates to be in the range [0, 2**31 - 1].
        range_check(
            ab,
            &[4, 4, 4, 4],
            &[limbs[3].clone(), limbs[7].clone(), limbs[11].clone(), limbs[15].clone()],
        );

        // Constrain the coordinates to not be PRIME = 2**31 - 1.
        let m31_limb_sum = const_expr!(3 * ((1 << 9) - 1) + (1 << 4) - 1);
        let deltas: [FeltExpr; 4] = from_fn(|i| {
            limbs[i * 4].clone()
                + limbs[i * 4 + 1].clone()
                + limbs[i * 4 + 2].clone()
                + limbs[i * 4 + 3].clone()
                - m31_limb_sum.clone()
        });
        let delta_ab = deltas[0].clone() * deltas[1].clone();
        let delta_cd = deltas[2].clone() * deltas[3].clone();
        let delta_prefix =
            addr.extra_info.clone().map(|s| format!("{s}_delta_")).unwrap_or("delta_".to_string());
        let delta_ab_inv =
            ab.deduce(&mut delta_ab.clone().inverse(), &format!("{delta_prefix}ab_inv"));
        let delta_cd_inv =
            ab.deduce(&mut delta_cd.clone().inverse(), &format!("{delta_prefix}cd_inv"));
        ab.constrain(
            delta_ab * delta_ab_inv - const_expr!(1),
            &format!("{delta_prefix}ab doesn't equal 0"),
        );
        ab.constrain(
            delta_cd * delta_cd_inv - const_expr!(1),
            &format!("{delta_prefix}cd doesn't equal 0"),
        );

        // Compute the M31 coordinates and output them along with the id.
        let coordinates = from_fn(|i| {
            limbs[i * 4].clone()
                + (limbs[i * 4 + 1].clone() * const_expr!(1 << 9))
                + (limbs[i * 4 + 2].clone() * const_expr!(1 << 18))
                + (limbs[i * 4 + 3].clone() * const_expr!(1 << 27))
        });
        (coordinates, id)
    }
}
