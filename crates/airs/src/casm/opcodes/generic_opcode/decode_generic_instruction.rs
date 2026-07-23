use std::collections::BTreeSet;

use air_infra::casm_state::CasmAddress;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use serde::Serialize;

use super::generic_opcode::*;
use crate::casm::common::*;
use crate::casm::decode_instruction::decode_inst::*;

#[derive(Clone, Debug, Serialize)]
pub struct DecodeGenericInstruction {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

// Returns a Decoded instance containing the values of the control flags and the offsets.
// Adds the relevant constraints to assert a defined behavior.
impl AirFn for DecodeGenericInstruction {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = ([FeltExpr; GENERIC_FLAGS_SIZE], [FeltExpr; 3]);

    fn output_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        let mut result = vec![];
        for name in GENERIC_FLAG_NAMES.iter() {
            result.push(Some(name.to_string()));
        }
        result.push(Some("offset0".to_string()));
        result.push(Some("offset1".to_string()));
        result.push(Some("offset2".to_string()));
        Some(result)
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), pc: Self::In) -> Self::Out {
        let (offsets, flags, _) = air_builder.call(
            &DecodeInstruction {
                const_offsets: [None, None, None],
                const_flags: Default::default(),
                const_opcode_extension: Some(OpcodeExtension::Stone),
                flag_sets_of_sum_1: BTreeSet::new(),
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        let mut generic_flags_vec = flags.to_vec();
        // op1_base_op0 = 1 iff FLAG_OP1_IMM = 0 and FLAG_OP1_BASE_FP = 0 and FLAG_OP1_BASE_AP = 0
        let op1_base_op0 = air_builder.let_(
            const_expr!(1)
                - generic_flags_vec[FLAG_OP1_IMM_INDEX].clone()
                - generic_flags_vec[FLAG_OP1_BASE_FP_INDEX].clone()
                - generic_flags_vec[FLAG_OP1_BASE_AP_INDEX].clone(),
            "op1_base_op0",
        );
        // Assert op1_src = 0 / 1 / 2 / 4
        air_builder.constrain(
            op1_base_op0.clone() * (const_expr!(1) - op1_base_op0.clone()),
            "op1_src is 0, 1, 2, or 4",
        );
        assert_eq!(flags.len(), FLAG_OP1_BASE_OP0_INDEX);
        generic_flags_vec.push(op1_base_op0);

        // res_op1 = 1 iff FLAG_RES_ADD = 0 and FLAG_RES_MUL = 0 and FLAG_PC_UPDATE_JNZ = 0
        let res_op1 = air_builder.let_(
            const_expr!(1)
                - generic_flags_vec[FLAG_RES_ADD_INDEX].clone()
                - generic_flags_vec[FLAG_RES_MUL_INDEX].clone()
                - generic_flags_vec[FLAG_PC_UPDATE_JNZ_INDEX].clone(),
            "res_op1",
        );
        // Assert res_logic = 0 / 1 / 2
        air_builder.constrain(
            res_op1.clone() * (const_expr!(1) - res_op1.clone()),
            "res_logic is 0, 1, or 2",
        );
        assert_eq!(generic_flags_vec.len(), FLAG_RES_OP1_INDEX);
        generic_flags_vec.push(res_op1.clone());

        // pc_update_regular = 1 iff FLAG_PC_UPDATE_JUMP = 0 and FLAG_PC_UPDATE_JUMP_REL = 0 and
        // FLAG_PC_UPDATE_JNZ = 0
        let pc_update_regular = air_builder.let_(
            const_expr!(1)
                - generic_flags_vec[FLAG_PC_UPDATE_JUMP_INDEX].clone()
                - generic_flags_vec[FLAG_PC_UPDATE_JUMP_REL_INDEX].clone()
                - generic_flags_vec[FLAG_PC_UPDATE_JNZ_INDEX].clone(),
            "pc_update_regular",
        );
        // Assert pc_update = 0 / 1 / 2 / 4
        air_builder.constrain(
            pc_update_regular.clone() * (const_expr!(1) - pc_update_regular.clone()),
            "pc_update is 0, 1, 2, or 4",
        );
        assert_eq!(generic_flags_vec.len(), FLAG_PC_UPDATE_REGULAR_INDEX);
        generic_flags_vec.push(pc_update_regular);

        // ap_update_regular = 1 iff FLAG_AP_UPDATE_ADD = 0 and FLAG_AP_UPDATE_ADD_1 = 0 and
        // FLAG_OPCODE_CALL = 0
        let ap_update_regular = air_builder.let_(
            const_expr!(1)
                - generic_flags_vec[FLAG_AP_UPDATE_ADD_INDEX].clone()
                - generic_flags_vec[FLAG_AP_UPDATE_ADD_1_INDEX].clone()
                - generic_flags_vec[FLAG_OPCODE_CALL_INDEX].clone(),
            "ap_update_regular",
        );
        // Assert ap_update = 0 / 1 / 2 / 4
        air_builder.constrain(
            ap_update_regular.clone() * (const_expr!(1) - ap_update_regular.clone()),
            "ap_update is 0, 1, 2, or 4",
        );

        // fp_update_regular = 1 iff FLAG_OPCODE_CALL = 0 and FLAG_OPCODE_RET = 0
        let fp_update_regular = air_builder.let_(
            const_expr!(1)
                - generic_flags_vec[FLAG_OPCODE_CALL_INDEX].clone()
                - generic_flags_vec[FLAG_OPCODE_RET_INDEX].clone(),
            "fp_update_regular",
        );
        // Assert opcode = 0 / 1 / 2 / 4
        air_builder.constrain(
            fp_update_regular.clone() * (const_expr!(1) - fp_update_regular.clone()),
            "opcode is 0, 1, 2, or 4",
        );
        assert_eq!(generic_flags_vec.len(), FLAG_FP_UPDATE_REGULAR_INDEX);
        generic_flags_vec.push(fp_update_regular);

        // Push instruction size
        assert_eq!(generic_flags_vec.len(), INSTRUCTION_SIZE_INDEX);
        generic_flags_vec.push(const_expr!(1) + generic_flags_vec[FLAG_OP1_IMM_INDEX].clone());
        let flags_array: [FeltExpr; GENERIC_FLAGS_SIZE] =
            generic_flags_vec.try_into().expect("Invalid generic flags vector size");
        (flags_array, offsets)
    }
}
