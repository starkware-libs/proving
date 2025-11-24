use compiled_casm_air::compiled_structs::TraceType;
use serde::Serialize;

use super::partial_ec_mul::*;
use super::points_table::*;
use super::read_split::*;
use crate::airs::felt252_utils::verify_reduced252::*;
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;

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
        ([a, b], output_id): Self::In,
    ) -> Self::Out {
        let (a_high, [a_low, a_full]) = air_builder.call(
            &ReadSplit {
                memory: self.memory.clone(),
            },
            a,
        );

        let (b_high, [b_low, b_full]) = air_builder.call(
            &ReadSplit {
                memory: self.memory.clone(),
            },
            b,
        );

        // Verify a, b < P
        air_builder.call(&VerifyReduced252 {}, a_full);
        air_builder.call(&VerifyReduced252 {}, b_full);

        // sum_0 = P_SHIFT * (2 * NUM_WINDOWS + 1) + a_high * P1 + b_high * P3
        let sum_0 = air_builder.lookup_call(
            &PedersenPointsTable {},
            [const_expr!(P_13_SECTION_START) + b_high * const_expr!(16) + a_high],
            (),
        );

        // sum_1 = sum_0 + a_low * P_0 - P_SHIFT * NUM_WINDOWS
        let (_, sum_1) = air_builder.chain_lookup_call::<PartialECMulState>(
            &PartialECMul {},
            (felt252_to_double_limbs(a_low), sum_0),
            0,
            NUM_WINDOWS,
        );
        // sum_2 = sum_1 + b_low * P_2 - P_SHIFT * NUM_WINDOWS
        let (_, sum_2) = air_builder.chain_lookup_call::<PartialECMulState>(
            &PartialECMul {},
            (felt252_to_double_limbs(b_low), sum_1),
            NUM_WINDOWS,
            NUM_WINDOWS,
        );

        self.memory
            .mem_verify_known_id(air_builder, &output_id, sum_2[0].clone());
    }
}
