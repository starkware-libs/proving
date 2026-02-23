use air_common::TraceType;
use serde::Serialize;

use super::partial_ec_mul_generic::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::seq::*;
use crate::airs::felt252_utils::felt252_packing27::*;
use crate::airs::felt252_utils::verify_reduced252::*;
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::verify::*;
use crate::core::public_params::PublicParam;

// Each ECOp instance consists of 7 memory cells:
// 5 inputs [p.x, p.y, q.x, q.y, m], and 2 outputs [res.x, res.y].
const ECOP_INSTANCE_SIZE: u32 = 7;

/// The builtin enforces that `p + m * q = res`, where, for addr = ECOpBuiltinSegmentStart + Seq*7,
///  - p = Point(mem[addr : addr + 2]),
///  - q = Point(mem[addr + 2 : addr + 4]),
///  - m = mem[addr + 4], and
///  - res = Point(mem[addr + 4 : addr + 6]).
///
/// The builtin assumes but does not verify that `p, q` are indeed on the curve.
#[derive(Debug, Serialize, Default)]
pub struct ECOpBuiltin {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for ECOpBuiltin {
    type ExtIn = ();
    type In = ();
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), _: ()) -> Self::Out {
        let instance_num = air_builder.call_external_table(&Seq {});
        let segment_start = air_builder.get_public_param(PublicParam::ECOpBuiltinSegmentStart);
        let instance_addr = air_builder.let_(
            instance_num * const_expr!(ECOP_INSTANCE_SIZE) + segment_start,
            "instance_addr",
        );

        // Read the inputs.
        let [p_x, p_y, q_x, q_y, m] = ["p_x", "p_y", "q_x", "q_y", "m"]
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let address = CasmAddress::new(instance_addr.clone() + const_expr!(i), name);
                self.memory.read_felt252(air_builder, address)
            })
            .collect::<Vec<Felt252Expr>>()
            .try_into()
            .unwrap();

        air_builder.call(&VerifyReduced252 {}, m.clone());

        let packed_m = felt252_pack_into27(m);

        let (res_m, _, [res_x, res_y], _) = air_builder
            .chain_lookup_call::<PartialECMulGenericState>(
                &PartialECMulGeneric {},
                (packed_m, [q_x, q_y], [p_x, p_y], const_expr!(26)),
                0,
                252,
            );

        // Constrain the least limb of the returned coefficient to be 0.
        air_builder.constrain(res_m.get_felt(0), "final m is zero");

        air_builder.call(
            &MemVerify {
                memory: self.memory.clone(),
            },
            (
                CasmAddress::new(instance_addr.clone() + const_expr!(5), "res_x"),
                res_x,
            ),
        );
        air_builder.call(
            &MemVerify {
                memory: self.memory.clone(),
            },
            (
                CasmAddress::new(instance_addr.clone() + const_expr!(6), "res_y"),
                res_y,
            ),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }
}
