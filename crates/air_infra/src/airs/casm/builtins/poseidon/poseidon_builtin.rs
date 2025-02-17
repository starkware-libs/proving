use compiled_casm_air::public_params::PublicParam;
use inst_def::InstDef;

use super::poseidon_permutation::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::seq::*;
use crate::airs::felt252_utils::felt252_packing27::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252width27_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::verify::*;

// Each Poseidon operation consists of 6 cells (3 inputs and 3 outputs, each being a single state).
pub const CELLS_PER_POSEIDON: u32 = 6;

/// The builtin enforces that
///   PoseidonHadesPermutation(mem[addr : addr + 3]) = mem[addr + 3 : addr + 6]
/// where addr = PoseidonBuiltinSegmentStart + Seq * 6.
#[derive(Debug, InstDef, Default)]
pub struct PoseidonBuiltin {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for PoseidonBuiltin {
    type ExtIn = ();
    type In = ();
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), _: ()) -> Self::Out {
        let instance_num = air_builder.call_external_table(&Seq {});
        let segment_start = air_builder.get_public_param(PublicParam::PoseidonBuiltinSegmentStart);

        // TODO(DanC): Allow and use direct access to memory in packed form, to save trace columns.
        let verify_felt252 = MemVerify {
            memory: self.memory.clone(),
        };

        let input_state: [Felt252Width27Expr; 3] = std::array::from_fn(|i| {
            felt252_pack_into27(self.memory.read_felt252(
                air_builder,
                CasmAddress::new(
                    get_addr(segment_start.clone(), instance_num.clone(), i as u32),
                    &format!("input_state_{}", i),
                ),
            ))
        });

        let output_state = air_builder.call(&PoseidonHadesPermutation {}, input_state);

        for (i, part) in output_state.into_iter().enumerate() {
            let unpacked_part = air_builder.call(
                &Felt252UnpackFrom27 {
                    range_check_output: false,
                },
                part,
            );
            air_builder.call(
                &verify_felt252,
                (
                    CasmAddress::new(
                        get_addr(segment_start.clone(), instance_num.clone(), i as u32 + 3),
                        &format!("output_state_{}", i),
                    ),
                    unpacked_part,
                ),
            );
        }
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }
}

pub fn get_addr(segment_start: FeltExpr, instance_num: FeltExpr, offset: u32) -> FeltExpr {
    segment_start + instance_num * const_expr!(CELLS_PER_POSEIDON) + const_expr!(offset)
}
