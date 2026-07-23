use air_common::TraceType;
use air_infra::const_felt252_expr_from_felt252;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::felt252_id_memory::memory::{CasmId, Felt252IdMemory};
use air_infra::felt252_id_memory::read_positive::ReadPositiveKnownId;
use serde::Serialize;

use super::partial_ec_mul::*;
use crate::casm::builtins::ec_utils::utils::*;
use crate::felt252_utils::verify_reduced252::*;

#[derive(Debug, Default, Serialize)]
pub struct PedersenAggregator<const NUM_WINDOWS: usize> {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
    window_bits: usize,
}

impl<const NUM_WINDOWS: usize> PedersenAggregator<NUM_WINDOWS> {
    pub fn new(memory: Felt252IdMemory) -> Self {
        assert_eq!(252 % NUM_WINDOWS, 0);
        Self { memory, window_bits: 252 / NUM_WINDOWS }
    }
}

impl<const NUM_WINDOWS: usize> AirFn for PedersenAggregator<NUM_WINDOWS> {
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
        let a_full = air_builder
            .call(&ReadPositiveKnownId { num_bits: 252, memory: self.memory.clone() }, a_id);

        let b_full = air_builder
            .call(&ReadPositiveKnownId { num_bits: 252, memory: self.memory.clone() }, b_id);

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
        let (_, sum_1) = air_builder.chain_lookup_call::<PartialECMulState<NUM_WINDOWS>>(
            &PartialECMul::new(),
            (felt252_to_limbs(a_full), sum_0),
            0,
            NUM_WINDOWS,
        );
        // sum_2 = sum_1 + b_low * P_2 + b_high * P_3 - P_SHIFT * NUM_WINDOWS
        let (_, sum_2) = air_builder.chain_lookup_call::<PartialECMulState<NUM_WINDOWS>>(
            &PartialECMul::new(),
            (felt252_to_limbs(b_full), sum_1),
            NUM_WINDOWS,
            NUM_WINDOWS,
        );

        self.memory.mem_verify_known_id(air_builder, &output_id, sum_2[0].clone());
    }
}
