use compiled_casm_air::compiled_structs::TraceType;
use compiled_casm_air::public_params::PublicParam;
use serde::Serialize;

use super::partial_ec_mul::*;
use super::points_table::*;
use super::read_split::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::seq::*;
use crate::airs::felt252_utils::verify_reduced252::*;
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::verify::*;

const PEDERSEN_INSTANCE_SIZE: u32 = 3;

#[derive(Debug, Default, Serialize)]
pub struct PedersenBuiltin {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for PedersenBuiltin {
    type ExtIn = ();
    type In = ();
    type Out = ();

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), _input: ()) -> Self::Out {
        let instance_num = air_builder.call_external_table(&Seq {});
        let segment_start = air_builder.get_public_param(PublicParam::PedersenBuiltinSegmentStart);
        let instance_addr = air_builder.let_(
            instance_num * const_expr!(PEDERSEN_INSTANCE_SIZE) + segment_start,
            "instance_addr",
        );
        let (a_high, [a_low, a_full]) = air_builder.call(
            &ReadSplit {
                memory: self.memory.clone(),
            },
            CasmAddress::new(instance_addr.clone(), "pedersen_a"),
        );

        let (b_high, [b_low, b_full]) = air_builder.call(
            &ReadSplit {
                memory: self.memory.clone(),
            },
            CasmAddress::new(instance_addr.clone() + const_expr!(1), "pedersen_b"),
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

        air_builder.call(
            &MemVerify {
                memory: self.memory.clone(),
            },
            (
                CasmAddress::new(instance_addr + const_expr!(2), "pedersen_result"),
                sum_2[0].clone(),
            ),
        );
    }
}
