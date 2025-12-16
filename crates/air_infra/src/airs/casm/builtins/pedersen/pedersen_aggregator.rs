use compiled_casm_air::compiled_structs::TraceType;
use serde::Serialize;

use super::partial_ec_mul::*;
use super::points_table::*;
use super::utils::*;
use crate::airs::felt252_utils::verify_reduced252::*;
use crate::const_felt252_expr_from_felt252;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::read_positive::*;

#[derive(Debug, Default, Serialize)]
pub struct PedersenAggregator {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for PedersenAggregator {
    type ExtIn = ();
    type In = ([CasmId; 2], CasmId);
    type Out = ();

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        ([a_id, b_id], output_id): Self::In,
    ) -> Self::Out {
        let a_full = air_builder.call(
            &ReadPositiveKnownId {
                num_bits: 252,
                memory: self.memory.clone(),
            },
            a_id,
        );

        let b_full = air_builder.call(
            &ReadPositiveKnownId {
                num_bits: 252,
                memory: self.memory.clone(),
            },
            b_id,
        );

        // Verify a, b < P
        air_builder.call(&VerifyReduced252 {}, a_full.clone());
        air_builder.call(&VerifyReduced252 {}, b_full.clone());

        // sum_0 = P_SHIFT * (2 * NUM_WINDOWS + 1)
        let sum_0_pt = ec_mul(&P_SHIFT, 2 * NUM_WINDOWS + 1);
        let sum_0 = [
            const_felt252_expr_from_felt252!(sum_0_pt.x),
            const_felt252_expr_from_felt252!(sum_0_pt.y),
        ];

        // sum_1 = sum_0 + a_low * P_0 + a_high * P_1 - P_SHIFT * NUM_WINDOWS
        let (_, sum_1) = air_builder.chain_lookup_call::<PartialECMulState>(
            &PartialECMul {},
            (felt252_to_double_limbs(a_full), sum_0),
            0,
            NUM_WINDOWS,
        );
        // sum_2 = sum_1 + b_low * P_2 + b_high * P_3 - P_SHIFT * NUM_WINDOWS
        let (_, sum_2) = air_builder.chain_lookup_call::<PartialECMulState>(
            &PartialECMul {},
            (felt252_to_double_limbs(b_full), sum_1),
            NUM_WINDOWS,
            NUM_WINDOWS,
        );

        self.memory
            .mem_verify_known_id(air_builder, &output_id, sum_2[0].clone());
    }
}
