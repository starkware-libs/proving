use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};

use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::variables::*;
use crate::{const_expr, const_u16_expr};

#[derive(Debug, Serialize)]
pub struct ReadSplit {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

// Read the felt252 value for the given ID and return it split into two parts: the least
// significant 248 bits (low) and the most significant 4 bits (high). Also returns the
// original (non-split) value.
impl AirFn for ReadSplit {
    type ExtIn = ();
    type In = CasmId;
    type Out = (FeltExpr, [Felt252Expr; 2]); // [high, low, original (high << 248 + low)]

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("id".to_string())])
    }

    fn output_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("high".to_string()),
            Some("low".to_string()),
            Some("original".to_string()),
        ])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), id: Self::In) -> Self::Out {
        const LOW_BITS_IN_MS_LIMB: u16 =
            (248 - (FELT252_N_WORDS - 1) * FELT252_BITS_PER_WORD) as u16;
        let mut value = self.memory.read_unverified_known_id(air_builder, &id);

        // Deduce the low limbs as-is
        for i in 0..(FELT252_N_WORDS - 1) {
            air_builder.deduce(value.get_felt_mut(i), &format!("value_limb_{}", i));
        }

        // Deduce the most significant limb split into two parts
        let ms_limb = air_builder.let_for_deduction(
            UInt16Expr::from(value.get_felt(FELT252_N_WORDS - 1)),
            "ms_limb",
        );
        let ms_limb_low = air_builder.deduce_air_var(
            ms_limb.clone() & const_u16_expr!((1 << LOW_BITS_IN_MS_LIMB) - 1),
            "ms_limb_low",
        );
        let ms_limb_high = air_builder.deduce_air_var(
            ms_limb >> const_u16_expr!(LOW_BITS_IN_MS_LIMB),
            "ms_limb_high",
        );

        // Range check the parts
        range_check(
            air_builder,
            &[
                LOW_BITS_IN_MS_LIMB,
                TryInto::<u16>::try_into(FELT252_BITS_PER_WORD).unwrap() - LOW_BITS_IN_MS_LIMB,
            ],
            &[ms_limb_low.as_felt(), ms_limb_high.as_felt()],
        );

        // Build the original value from the deduced parts and verify that it is indeed the value
        // in the memory.
        let mut memory_value_felts: Vec<_> = value.as_felts()[0..FELT252_N_WORDS - 1].into();
        memory_value_felts.push(
            ms_limb_high.as_felt() * const_expr!(1 << LOW_BITS_IN_MS_LIMB) + ms_limb_low.as_felt(),
        );

        self.memory
            .mem_verify_known_id(air_builder, &id, memory_value_felts.clone().into());

        // Build the low and high parts from the deduced felts.
        let mut low_felts: Vec<_> = value.as_felts()[0..FELT252_N_WORDS - 1].into();
        low_felts.push(ms_limb_low.as_felt());

        let high_felt = ms_limb_high.as_felt();

        (high_felt, [low_felts.into(), memory_value_felts.into()])
    }
}
