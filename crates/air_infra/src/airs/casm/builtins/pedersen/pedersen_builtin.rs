use compiled_casm_air::public_params::PublicParam;
use inst_def::InstDef;

use super::partial_ec_mul::*;
use super::points_table::*;
use super::read_split::*;
use super::utils::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::seq::*;
use crate::airs::felt252_utils::verify_reduced252::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::verify::*;
use crate::{const_expr, const_felt252_expr_from_felt252};

const PEDERSEN_INSTANCE_SIZE: u32 = 3;

#[derive(Debug, Default, InstDef)]
pub struct PedersenBuiltin {
    #[instdef(skip)]
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
        let [a_low, a_high, a_full] = air_builder.call(
            &ReadSplit {
                memory: self.memory.clone(),
            },
            CasmAddress::new(instance_addr.clone(), "pedersen_a"),
        );

        let [b_low, b_high, b_full] = air_builder.call(
            &ReadSplit {
                memory: self.memory.clone(),
            },
            CasmAddress::new(instance_addr.clone() + const_expr!(1), "pedersen_b"),
        );

        // Verify a, b < P
        air_builder.call(&VerifyReduced252 {}, a_full);
        air_builder.call(&VerifyReduced252 {}, b_full);

        // sum_0 = P_SHIFT * (2 * NUM_WINDOWS + 2 + 1)
        let sum_0 = ec_mul(&P_SHIFT, 2 * NUM_WINDOWS + 2 + 1);
        let sum_0_x = const_felt252_expr_from_felt252!(sum_0.x);
        let sum_0_y = const_felt252_expr_from_felt252!(sum_0.y);

        // sum_1 = sum_0 + a_low * P_0 - P_SHIFT * NUM_WINDOWS
        let (_, _, sum_1) = air_builder.chain_lookup_call::<PartialECMulState>(
            &PartialECMul {},
            (
                const_expr!(P_0_SECTION_START),
                felt252_to_double_limbs(a_low),
                [sum_0_x, sum_0_y],
            ),
            0,
            NUM_WINDOWS,
        );

        // sum_2 = sum_1 + a_high * P_1 - P_SHIFT
        let (_, _, sum_2) = air_builder.chain_lookup_call::<PartialECMulState>(
            &PartialECMul {},
            (
                const_expr!(P_1_SECTION_START),
                felt252_to_double_limbs(a_high),
                sum_1,
            ),
            0,
            1,
        );

        // sum_3 = sum_2 + b_low * P_2 - P_SHIFT * NUM_WINDOWS
        let (_, _, sum_3) = air_builder.chain_lookup_call::<PartialECMulState>(
            &PartialECMul {},
            (
                const_expr!(P_2_SECTION_START),
                felt252_to_double_limbs(b_low),
                sum_2,
            ),
            0,
            NUM_WINDOWS,
        );

        // sum_4 = sum_3 + b_high * P_3 - P_SHIFT
        let (_, _, sum_4) = air_builder.chain_lookup_call::<PartialECMulState>(
            &PartialECMul {},
            (
                const_expr!(P_3_SECTION_START),
                felt252_to_double_limbs(b_high),
                sum_3,
            ),
            0,
            1,
        );

        air_builder.call(
            &MemVerify {
                memory: self.memory.clone(),
            },
            (
                CasmAddress::new(instance_addr + const_expr!(2), "pedersen_result"),
                sum_4[0].clone(),
            ),
        );
    }
}
