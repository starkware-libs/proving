use std::collections::{BTreeMap, BTreeSet};

use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_u16_expr};
use serde::Serialize;

use super::verify_inst::*;
use crate::casm::common::*;

#[derive(Clone, Debug, Serialize)]
pub struct DecodeInstruction {
    pub const_offsets: [Option<i16>; 3], // off_0, off_1, off_2
    pub const_flags: Flags,
    pub const_opcode_extension: Option<OpcodeExtension>,
    pub flag_sets_of_sum_1: BTreeSet<BTreeSet<usize>>,
    #[serde(skip)]
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
// The field flag_sets_of_sum_1 is a set of sets of indexes of flags that should sum to 1. Each such
// set is expected to be non-empty. The last flag of each set is not deduced and is set to 1 minus
// the sum of the other flags in the set. If there are more than 2 flags in the set, the last flag
// is constrained to be a bit.
impl AirFn for DecodeInstruction {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = ([FeltExpr; 3], [FeltExpr; 15], FeltExpr);

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("pc".to_string())])
    }

    fn output_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("offset0".to_string()),
            Some("offset1".to_string()),
            Some("offset2".to_string()),
            Some("dst_base_fp".to_string()),
            Some("op0_base_fp".to_string()),
            Some("op1_imm".to_string()),
            Some("op1_base_fp".to_string()),
            Some("op1_base_ap".to_string()),
            Some("res_add".to_string()),
            Some("res_mul".to_string()),
            Some("pc_update_jump".to_string()),
            Some("pc_update_jump_rel".to_string()),
            Some("pc_update_jnz".to_string()),
            Some("ap_update_add".to_string()),
            Some("ap_update_add_1".to_string()),
            Some("opcode_call".to_string()),
            Some("opcode_ret".to_string()),
            Some("opcode_assert_eq".to_string()),
            Some("opcode_extension".to_string()),
        ])
    }

    fn call(&self, ab: &mut AirBuilder, _: (), pc: Self::In) -> Self::Out {
        // Decode the instruction without verification
        let (instruction, _) = self.memory.read_unverified(ab, &pc);
        let ([mut off0, mut off1, mut off2, flags], mut opcode_extension) =
            Self::decode_instruction(instruction);

        // Deduce the non-constant offsets
        off0 = if let Some(off) = self.const_offsets[0] {
            const_u16_expr!(offset_as_u16(off))
        } else {
            ab.deduce_air_var(off0, "offset0")
        };

        off1 = if let Some(off) = self.const_offsets[1] {
            const_u16_expr!(offset_as_u16(off))
        } else {
            ab.deduce_air_var(off1, "offset1")
        };

        off2 = if let Some(off) = self.const_offsets[2] {
            const_u16_expr!(offset_as_u16(off))
        } else {
            ab.deduce_air_var(off2, "offset2")
        };

        // Build a map that maps the last flag of each set in flag_sets_of_sum_1 to the rest of that
        // set. We will build the flags vector in ascending order of indexes, which is the same
        // order as each BTreeSet. Thus, when we reach an index which is a key of this map,
        // it will map to a set of smaller indexes, which are already in the flag vector. We can
        // then infer it to be 1 minus the sum of the flags in the set it maps to.
        let err_msg = "Expected sets in flag_sets_of_sum_1 to be non-empty";
        let last_to_rest = self
            .flag_sets_of_sum_1
            .iter()
            .map(|set| {
                let mut rest = set.clone();
                let last = rest.take(set.iter().last().expect(err_msg)).expect(err_msg);
                (last, rest)
            })
            .collect::<BTreeMap<_, _>>();

        // Deduce the non-constant flags and infer the last flag of each set.
        let mut flags_vec: Vec<FeltExpr> = vec![];
        for (i, flag) in self.const_flags.to_arr().iter().enumerate() {
            let flag_to_push = if last_to_rest.contains_key(&i) {
                // Inferred flag - the last flag of each set is given the value of 1 minus the sum
                // of the other flags. If there are more than 2 flags in the set, it
                // needs to be constrained to be a bit.
                let mut inferred_flag = last_to_rest[&i]
                    .iter()
                    .map(|&j| flags_vec[j].clone())
                    .fold(const_expr!(1), |acc, flag| acc - flag);

                if last_to_rest[&i].len() > 1 {
                    // The expression for the inferred variable contains multiple operations.
                    // Put it in an intermediate to simplify the "is a bit" constraint below.
                    inferred_flag = ab.let_(inferred_flag, FLAG_NAMES[i]);

                    ab.constrain(
                        inferred_flag.clone() * (const_expr!(1) - inferred_flag.clone()),
                        &format!("Flag {} is a bit", FLAG_NAMES[i]),
                    );
                }
                inferred_flag
            } else if let Some(flag) = flag {
                // Const flag - doesn't need to be constrained to be a bit.
                const_expr!(*flag as u32)
            } else {
                // Deduced flag - read from the instruction and deduced.
                // Needs to be constrained to be a bit.
                let flag = ab.deduce_air_var(
                    (flags.clone() >> const_u16_expr!(i as u16)) & const_u16_expr!(1),
                    FLAG_NAMES[i],
                );
                ab.constrain(
                    flag.as_felt() * (const_expr!(1) - flag.as_felt()),
                    &format!("Flag {} is a bit", FLAG_NAMES[i]),
                );
                flag.as_felt()
            };
            flags_vec.push(flag_to_push);
        }
        let flags_array: [FeltExpr; 15] = flags_vec.try_into().expect("Expected 15 flags");

        // Deduce opcode extension if is not a constant
        if let Some(constant) = self.const_opcode_extension {
            opcode_extension = constant.into();
        } else {
            ab.deduce(&mut opcode_extension, "opcode_extension");
        };
        // Construct the felts holding the flags
        let [felt5_high, felt6] = Self::flags_to_felts(flags_array.clone());

        // Verify the instruction
        ab.lookup_call(
            &VerifyInstruction { memory: self.memory.clone() },
            (),
            (
                pc.clone(),
                [off0.as_felt(), off1.as_felt(), off2.as_felt()],
                [felt5_high, felt6],
                opcode_extension.clone(),
            ),
        );

        (
            [
                offset_as_signed(off0.as_felt()),
                offset_as_signed(off1.as_felt()),
                offset_as_signed(off2.as_felt()),
            ],
            flags_array,
            opcode_extension,
        )
    }

    fn description(&self) -> String {
        "Decode Instruction".to_string()
    }
}
