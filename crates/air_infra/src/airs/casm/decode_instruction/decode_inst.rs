use inst_def::InstDef;

use super::super::casm_state::*;
use super::super::common::*;
use super::verify_inst::*;
// Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::felt252_id_memory::memory::*;

#[derive(Clone, Debug, InstDef)]
pub struct DecodeInstruction {
    pub const_offsets: [Option<i16>; 3], // off_0, off_1, off_2
    pub const_flags: Flags,
    pub const_opcode_extension: Option<OpcodeExtension>,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl DecodeInstruction {
    fn decode_instruction(inst: Felt252Expr) -> ([UInt16Expr; 4], FeltExpr) {
        let off0 = UInt16Expr::from(inst.get_felt(0))
            + ((UInt16Expr::from(inst.get_felt(1)) & const_u16_expr!(127)) << const_u16_expr!(9));

        let off1 = ((UInt16Expr::from(inst.get_felt(1)) >> const_u16_expr!(7))
            + (UInt16Expr::from(inst.get_felt(2)) << const_u16_expr!(2)))
            + ((UInt16Expr::from(inst.get_felt(3)) & const_u16_expr!(31)) << const_u16_expr!(11));

        let off2 = ((UInt16Expr::from(inst.get_felt(3)) >> const_u16_expr!(5))
            + (UInt16Expr::from(inst.get_felt(4)) << const_u16_expr!(4)))
            + ((UInt16Expr::from(inst.get_felt(5)) & const_u16_expr!(7)) << const_u16_expr!(13));

        let flags = (UInt16Expr::from(inst.get_felt(5)) >> const_u16_expr!(3))
            + (UInt16Expr::from(inst.get_felt(6)) << const_u16_expr!(6));

        let opcode_extension = inst.get_felt(7);

        ([off0, off1, off2, flags], opcode_extension)
    }

    pub fn flags_to_felts(flags: [FeltExpr; 15]) -> [FeltExpr; 2] {
        let mut felt5_high = const_expr!(0);
        for (i, flag) in flags.iter().enumerate().take(6) {
            felt5_high = felt5_high.clone() + (flag.clone() * const_expr!(1 << (i + 3)));
        }

        let mut felt6 = const_expr!(0);
        for (i, flag) in flags.into_iter().enumerate().skip(6) {
            felt6 = felt6.clone() + (flag * const_expr!(1 << (i - 6)));
        }

        [felt5_high, felt6]
    }
}

// Given the address of the instructions, reads the instruction and deduces the non-constant
// offsets and flags. Returns all offsets and flags (constants and deduced).
impl AirFn for DecodeInstruction {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = ([FeltExpr; 3], [FeltExpr; 15], FeltExpr);

    fn call(&self, ab: &mut AirBuilder, _: (), pc: Self::In) -> Self::Out {
        // Decode the instruction without verification
        let (instruction, _) = self.memory.read_unverified(ab, &pc);
        let ([mut off0, mut off1, mut off2, flags], mut opcode_extnesion) =
            Self::decode_instruction(instruction);

        // Deduce the non-constant offsets
        let off0_f = if let Some(off) = self.const_offsets[0] {
            const_expr!(offset_as_u16(off) as u32)
        } else {
            off0 = ab.let_for_deduction(off0, "offset0");
            ab.deduce(off0.as_felt_mut(), "offset0")
        };

        let off1_f = if let Some(off) = self.const_offsets[1] {
            const_expr!(offset_as_u16(off) as u32)
        } else {
            off1 = ab.let_for_deduction(off1, "offset1");
            ab.deduce(off1.as_felt_mut(), "offset1")
        };

        let off2_f = if let Some(off) = self.const_offsets[2] {
            const_expr!(offset_as_u16(off) as u32)
        } else {
            off2 = ab.let_for_deduction(off2, "offset2");
            ab.deduce(off2.as_felt_mut(), "offset2")
        };

        // Deduce the non-constant flags
        let flags_vec: [FeltExpr; 15] = self
            .const_flags
            .to_arr()
            .iter()
            .enumerate()
            .map(|(i, flag)| {
                if let Some(flag) = flag {
                    const_expr!(*flag as u32)
                } else {
                    let mut flag = ab.let_for_deduction(
                        (flags.clone() >> const_u16_expr!(i as u16)) & const_u16_expr!(1),
                        FLAG_NAMES[i],
                    );
                    let flag_deduced = ab.deduce(flag.as_felt_mut(), FLAG_NAMES[i]);
                    ab.constrain(
                        flag_deduced.clone() * (const_expr!(1) - flag_deduced.clone()),
                        &format!("Flag {} is a bit", FLAG_NAMES[i]),
                    );
                    flag_deduced
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("Expected 15 flags");

        // Deduce opcode extension if is not a constant
        if let Some(constant) = self.const_opcode_extension {
            opcode_extnesion = constant.into();
        } else {
            ab.deduce(&mut opcode_extnesion, "opcode_extension");
        };
        // Construct the felts holding the flags
        let [felt5_high, felt6] = Self::flags_to_felts(flags_vec.clone());

        // Verify the instruction
        ab.lookup_call(
            &VerifyInstruction {
                memory: self.memory.clone(),
            },
            (),
            (
                pc.clone(),
                [off0_f.clone(), off1_f.clone(), off2_f.clone()],
                [felt5_high, felt6],
                opcode_extnesion.clone(),
            ),
        );

        (
            [
                offset_as_signed(off0_f),
                offset_as_signed(off1_f),
                offset_as_signed(off2_f),
            ],
            flags_vec,
            opcode_extnesion,
        )
    }

    fn description(&self) -> String {
        "Decode Instruction".to_string()
    }
}
