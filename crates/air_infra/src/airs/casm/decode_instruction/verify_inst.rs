use inst_def::InstDef;
use prover_types::cpu::FELT252_BITS_PER_WORD;

use super::super::casm_state::*;
use super::encode_flags::*;
use super::encode_offsets::*;
use crate::airs::casm::common::*;
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

// |   9   | 9 |   9   | 9 |   9   | 9 |   9   | 9 | Felts in the instruction
// | - | 2 | 9 | 6 | 3 | 9 | 4 | 5 | 9 | 2 | 7 | 9 | Parts of offsets and flags
// |   2   |  15   |    16     |    16     |  16   | Offsets and flags
//
// Deduces and range checks the parts of each offset and flag.
// Reconstructs the instruction and verifies it against the memory.
impl AirFn for VerifyInstruction {
    type ExtIn = ();
    type In = (CasmAddress, [FeltExpr; 3], [FeltExpr; 15], FeltExpr);
    type Out = ();

    fn call(
        &self,
        ab: &mut AirBuilder,
        _: (),
        (pc, offsets, flags, opcode_extension): Self::In,
    ) -> Self::Out {
        assert_eq!(
            FELT252_BITS_PER_WORD, 9,
            "VerifyInstruction assumes there are 9 bits per felt in a felt252"
        );

        let [felt0, felt1, felt2, felt3, felt4, felt5_low] = ab.call(&EncodeOffsets {}, offsets);

        let [felt5_high, felt6] = ab.call(&EncodeFlags {}, flags);

        let felt5 = felt5_low + felt5_high;

        ab.constrain(
            (opcode_extension.clone() - OpcodeExtension::Stone.into())
                * (opcode_extension.clone() - OpcodeExtension::Blake.into())
                * (opcode_extension.clone() - OpcodeExtension::BlakeFinalize.into()),
            "OpcodeExtension enum has a valid value",
        );

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
            &MemVerify {
                memory: self.memory.clone(),
            },
            (
                CasmAddress::new(pc.var, "instruction"),
                expected_instruction,
            ),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
