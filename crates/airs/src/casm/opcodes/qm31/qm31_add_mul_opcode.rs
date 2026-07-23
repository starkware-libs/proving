use std::collections::BTreeSet;

use air_common::TraceType;
use air_infra::casm_state::{CasmAddress, CasmStateVar};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use serde::Serialize;

use super::qm31_read_reduced::*;
use crate::casm::common::*;
use crate::casm::decode_instruction::decode_inst::*;

/// The qm31_add_mul opcode.
/// Implements the packed reduced QM31 operations:
/// - [ap/fp + offset0] = QM31Add([ap/fp + offset1], [ap/fp + offset2])
/// - [ap/fp + offset0] = QM31Add([ap/fp + offset1], Imm)
/// - [ap/fp + offset0] = QM31Mul([ap/fp + offset1], [ap/fp + offset2])
/// - [ap/fp + offset0] = QM31Mul([ap/fp + offset1], Imm)
///
/// QM31 is the degree 4 extension field of M31.
/// We represent elements of QM31 by their M31 coordinates with respect to the basis (1, i, j, k)
/// where i is a square root of -1, j is a square root of 2+i and k=i*j.
/// Those 4 M31 coordinates are represented by 4 nonegative integers smaller than 2^31-1 (reduced
/// form) and packed into a single Felt252 in memory by each using 4 columns of 9 bits.
#[derive(Clone, Debug, Serialize)]
pub struct QM31AddMulOpcode {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl QM31AddMulOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: None,
            op0_base_fp: None,
            op1_imm: None,
            op1_base_fp: None,
            op1_base_ap: None,
            res_add: None,
            res_mul: None,
            pc_update_jump: Some(false),
            pc_update_jump_rel: Some(false),
            pc_update_jnz: Some(false),
            ap_update_add: Some(false),
            ap_update_add_1: None,
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(true),
        }
    }
}

impl AirFn for QM31AddMulOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        let const_offsets = [None, None, None];
        // Check the instruction.
        let ([offset0, offset1, offset2], flags, _) = ab.call(
            &DecodeInstruction {
                const_offsets,
                const_flags: self.get_flags(),
                const_opcode_extension: Some(OpcodeExtension::QM31Operation),
                flag_sets_of_sum_1: BTreeSet::from([
                    BTreeSet::from([
                        FLAG_OP1_IMM_INDEX,
                        FLAG_OP1_BASE_FP_INDEX,
                        FLAG_OP1_BASE_AP_INDEX,
                    ]),
                    BTreeSet::from([FLAG_RES_ADD_INDEX, FLAG_RES_MUL_INDEX]),
                ]),
                memory: self.memory.clone(),
            },
            casm_state.pc().clone(),
        );

        // Read the non-constant flags
        let flag_dst_base_fp = flags[FLAG_DST_BASE_FP_INDEX].clone();
        let flag_op0_base_fp = flags[FLAG_OP0_BASE_FP_INDEX].clone();
        let flag_op1_imm = flags[FLAG_OP1_IMM_INDEX].clone();
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP_INDEX].clone();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP_INDEX].clone();
        let flag_res_add = flags[FLAG_RES_ADD_INDEX].clone();
        let flag_res_mul = flags[FLAG_RES_MUL_INDEX].clone();
        let flag_ap_update_add_1 = flags[FLAG_AP_UPDATE_ADD_1_INDEX].clone();

        ab.constrain(
            flag_op1_imm.clone() * (offset2.clone() - const_expr!(1)),
            "Either flag op1_imm is off or offset2 is equal to 1",
        );

        let mem_dst_base = ab.assign(
            &mut (flag_dst_base_fp.clone() * casm_state.fp().var
                + (const_expr!(1) - flag_dst_base_fp) * casm_state.ap().var),
            "mem_dst_base",
        );
        let mem0_base = ab.assign(
            &mut (flag_op0_base_fp.clone() * casm_state.fp().var
                + (const_expr!(1) - flag_op0_base_fp) * casm_state.ap().var),
            "mem0_base",
        );
        let mem1_base = ab.assign(
            &mut (flag_op1_base_fp * casm_state.fp().var
                + flag_op1_base_ap * casm_state.ap().var
                + flag_op1_imm.clone() * casm_state.pc().var),
            "mem1_base",
        );

        let (dst, _) = ab.call(
            &QM31ReadReduced { memory: self.memory.clone() },
            CasmAddress::new(mem_dst_base + offset0, "dst"),
        );
        let (op0, _) = ab.call(
            &QM31ReadReduced { memory: self.memory.clone() },
            CasmAddress::new(mem0_base + offset1, "op0"),
        );
        let (op1, _) = ab.call(
            &QM31ReadReduced { memory: self.memory.clone() },
            CasmAddress::new(mem1_base + offset2, "op1"),
        );

        // When expending (a0+b0i+c0j+d0k) * (a1+b1i+c1j+d1k)
        // and regrouping as coordinates in (1, i, j, k) we arrive at the result
        // a0 * a1 - b0 * b1 + 2*(c0*c1 - d0*d1) - c0*d1 - d0*c1
        // + i*(a0 * b1 + b0 * a1 + 2*(c0*d1 + d0*c1) + c0*c1 - d0*d1)
        // + j*(a0 * c1 - b0 * d1 + c0 * a1 - d0 * b1)
        // + k*(a0 * d1 + b0 * c1 + c0 * b1 + d0 * a1)
        // Hence, the coordinates in mul_result are those of op0 * op1
        let mul_result = [
            op0[0].clone() * op1[0].clone() - op0[1].clone() * op1[1].clone()
                + const_expr!(2)
                    * (op0[2].clone() * op1[2].clone() - op0[3].clone() * op1[3].clone())
                - op0[2].clone() * op1[3].clone()
                - op0[3].clone() * op1[2].clone(),
            op0[0].clone() * op1[1].clone()
                + op0[1].clone() * op1[0].clone()
                + const_expr!(2)
                    * (op0[2].clone() * op1[3].clone() + op0[3].clone() * op1[2].clone())
                + op0[2].clone() * op1[2].clone()
                - op0[3].clone() * op1[3].clone(),
            op0[0].clone() * op1[2].clone() - op0[1].clone() * op1[3].clone()
                + op0[2].clone() * op1[0].clone()
                - op0[3].clone() * op1[1].clone(),
            op0[0].clone() * op1[3].clone()
                + op0[1].clone() * op1[2].clone()
                + op0[2].clone() * op1[1].clone()
                + op0[3].clone() * op1[0].clone(),
        ];

        for i in 0..4 {
            ab.constrain(
                dst[i].clone()
                    - mul_result[i].clone() * flag_res_mul.clone()
                    - (op0[i].clone() + op1[i].clone()) * flag_res_add.clone(),
                "dst equals (op0 * op1)*flag_res_mul + (op0 + op1)*(1-flag_res_mul)",
            );
        }

        // Calculate the next ap
        let next_ap = casm_state.ap().var + flag_ap_update_add_1;

        // Calculate the next pc
        let next_pc = casm_state.pc().var + const_expr!(1) + flag_op1_imm;

        CasmStateVar::new(next_pc, next_ap, casm_state.fp().var)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
