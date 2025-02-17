use std::array::from_fn;

use inst_def::InstDef;

use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::range_check::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::read_positive::*;
use crate::core::variables::*;

// The QM31ReadReduced inline AirFn.
// Reads a 144-bit value from the memory representing a packed QM31 element and outputs its M31
// coordinates and its id. Constrains the value to be in reduced form.
#[derive(Debug, InstDef, Default)]
pub struct QM31ReadReduced {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for QM31ReadReduced {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = ([FeltExpr; 4], FeltExpr);

    fn call(&self, ab: &mut AirBuilder, _: (), addr: Self::In) -> Self::Out {
        // Read the 144-bit value from the memory.
        let (value, id) = ab.call(
            &ReadPositive {
                num_bits: 4 * 4 * 9,
                memory: self.memory.clone(),
            },
            addr.clone(),
        );
        let limbs = value.as_felts();

        // Constrain the coordinates to be in the range [0, 2**31 - 1].
        range_check(
            ab,
            &[4, 4, 4, 4],
            &[
                limbs[3].clone(),
                limbs[7].clone(),
                limbs[11].clone(),
                limbs[15].clone(),
            ],
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
        let delta_prefix = addr
            .desc
            .clone()
            .map(|s| format!("{}_delta_", s))
            .unwrap_or("delta_".to_string());
        let delta_ab_inv = ab.deduce(
            &mut delta_ab.clone().inverse(),
            &format!("{}ab_inv", delta_prefix),
        );
        let delta_cd_inv = ab.deduce(
            &mut delta_cd.clone().inverse(),
            &format!("{}cd_inv", delta_prefix),
        );
        ab.constrain(
            delta_ab * delta_ab_inv - const_expr!(1),
            &format!("{}ab doesn't equal 0", delta_prefix),
        );
        ab.constrain(
            delta_cd * delta_cd_inv - const_expr!(1),
            &format!("{}cd doesn't equal 0", delta_prefix),
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
