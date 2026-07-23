use air_common::TraceType;
use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::felt252_id_memory::verify::MemVerify;
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_BITS_PER_WORD;

use super::encode_offsets::*;

#[derive(Clone, Debug, Serialize, Default)]
pub struct VerifyInstruction {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

// |   9   | 9 |   9   | 9 |   9   | 9 |   9   | 9 | Felts in the instruction
// | - | 2 | 9 | 6 | 3 | 9 | 4 | 5 | 9 | 2 | 7 | 9 | Parts of offsets and flags
// |   2   |  15   |    16     |    16     |  16   | Offsets and flags
//
// Deduces and range checks the parts of each offset and flag.
// Reconstructs the instruction and verifies it against the memory.
impl AirFn for VerifyInstruction {
    type ExtIn = ();
    type In = (CasmAddress, [FeltExpr; 3], [FeltExpr; 2], FeltExpr);
    type Out = ();

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("pc".to_string()),
            Some("offset0".to_string()),
            Some("offset1".to_string()),
            Some("offset2".to_string()),
            Some("inst_felt5_high".to_string()),
            Some("inst_felt6".to_string()),
            Some("opcode_extension".to_string()),
        ])
    }

    fn call(
        &self,
        ab: &mut AirBuilder,
        _: (),
        (pc, offsets, [felt5_high, felt6], opcode_extension): Self::In,
    ) -> Self::Out {
        assert_eq!(
            FELT252_BITS_PER_WORD, 9,
            "VerifyInstruction assumes there are 9 bits per felt in a felt252"
        );

        let [felt0, felt1, felt2, felt3, felt4, felt5_low] = ab.call(&EncodeOffsets {}, offsets);

        let felt5 = felt5_low + felt5_high;

        let expected_instruction = Felt252Expr::from(vec![
            felt0,
            felt1,
            felt2,
            felt3,
            felt4,
            felt5,
            felt6,
            opcode_extension,
        ]);

        ab.call(
            &MemVerify { memory: self.memory.clone() },
            (CasmAddress::new(pc.var, "instruction"), expected_instruction),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
