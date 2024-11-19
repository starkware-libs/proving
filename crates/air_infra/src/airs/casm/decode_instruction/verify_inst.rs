use inst_def::InstDef;
use prover_types::cpu::FELT252_BITS_PER_WORD;

use super::super::casm_state::*;
use super::encode_flags::*;
use super::encode_offsets::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::verify::*;

#[derive(Clone, Debug, InstDef, Default)]
pub struct VerifyInstruction {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

// | 9 |   9   | 9 |   9   | 9 |   9   | 9 | Felts in the instruction
// | 9 | 6 | 3 | 9 | 4 | 5 | 9 | 2 | 7 | 9 | Parts of offsets and flags
// |  15   |    16     |    16     |  16   | Offsets and flags
//
// Deduces and range checks the parts of each offset and flag.
// Reconstructs the instruction and verifies it against the memory.
impl AirFn for VerifyInstruction {
    type In = (CasmAddress, [FeltExpr; 3], [FeltExpr; 15]);
    type Out = ();

    fn call(&self, ab: &mut AirBuilder, (pc, offsets, flags): Self::In) -> Self::Out {
        assert_eq!(
            FELT252_BITS_PER_WORD, 9,
            "VerifyInstruction assumes there are 9 bits per felt in a felt252"
        );

        let [felt0, felt1, felt2, felt3, felt4, felt5_low] = ab.call(
            &EncodeOffsets {
                memory: self.memory.clone(),
            },
            offsets,
        );

        let [felt5_high, felt6] = ab.call(
            &EncodeFlags {
                memory: self.memory.clone(),
            },
            flags,
        );

        let felt5 = felt5_low + felt5_high;
        let expected_instruction =
            Felt252Expr::from(vec![felt0, felt1, felt2, felt3, felt4, felt5, felt6]);

        ab.call(
            &MemVerify {
                memory: self.memory.clone(),
            },
            (
                CasmAddress::new(pc.value.clone(), "instruction"),
                expected_instruction,
            ),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
