use crate::core::air_fn::*;
use crate::core::expressions::expr::GenericExprImpl;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::Felt252;
use crate::core::prover_types::FELT252_BITS_PER_WORD;
use crate::core::prover_types::P_FELTS;
use crate::core::variables::AirVar;
use crate::core::Felt;
use stwo_prover::core::fields::m31::M31;

use super::check_instruction::*;
use super::common::*;
use super::read_small_felt252::ReadSmallFelt252;

// Macros
use crate::const_expr;

pub const FLAG_OP1_BASE_OP0: usize = 15;
pub const FLAG_RES_OP1: usize = 16;
pub const FLAG_PC_UPDATE_REGULAR: usize = 17;
// pub const FLAG_AP_UPDATE_REGULAR: usize = 18;
pub const FLAG_FP_UPDATE_REGULAR: usize = 18;
pub const INSTRUCTION_SIZE: usize = 19;
pub const NUM_LIMBS_PER_ADDR: usize = 3;

/// Implements a generic Cairo0 instructions.
/// All the flags are read from the trace.

#[derive(Clone, Debug)]
pub struct GenericOpcode {
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl GenericOpcode {
    // Returns a Decoded instance containing the values of the control flags and the offsets.
    fn decode_instruction(
        &self,
        ab: &mut AirBuilder,
        pc: FeltExpr,
    ) -> (Vec<GenericExprImpl<Felt>>, [GenericExprImpl<M31>; 3]) {
        let (offsets, flags) = ab.call(
            &CheckInstruction {
                const_offsets: [None, None, None],
                const_flags: Default::default(),
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        let mut flags_as_felts = flags.iter().map(|f| f.as_felt()).collect::<Vec<_>>();
        // op1_base_op0 = 1 iff FLAG_OP1_IMM = 0 and FLAG_OP1_BASE_FP = 0 and FLAG_OP1_BASE_AP = 0
        let op1_base_op0 = const_expr!(1)
            - flags_as_felts[FLAG_OP1_IMM].clone()
            - flags_as_felts[FLAG_OP1_BASE_FP].clone()
            - flags_as_felts[FLAG_OP1_BASE_AP].clone();
        // Assert op1_src = 0 / 1 / 2 / 4
        ab.constrain(
            const_expr!(1)
                - op1_base_op0.clone()
                - flags_as_felts[FLAG_OP1_IMM].clone()
                - flags_as_felts[FLAG_OP1_BASE_FP].clone()
                - flags_as_felts[FLAG_OP1_BASE_AP].clone(),
        );
        flags_as_felts.push(op1_base_op0);

        // res_op1 = 1 iff FLAG_RES_ADD = 0 and FLAG_RES_MUL = 0
        let res_op1 = const_expr!(1)
            - flags_as_felts[FLAG_RES_ADD].clone()
            - flags_as_felts[FLAG_RES_MUL].clone();
        // Assert res_logic = 0 / 1 / 2
        ab.constrain(
            const_expr!(1)
                - res_op1.clone()
                - flags_as_felts[FLAG_RES_ADD].clone()
                - flags_as_felts[FLAG_RES_MUL].clone(),
        );
        flags_as_felts.push(res_op1);

        // pc_update_regular = 1 iff FLAG_PC_UPDATE_JUMP = 0 and FLAG_PC_UPDATE_JUMP_REL = 0 and FLAG_PC_UPDATE_JNZ = 0
        let pc_update_regular = const_expr!(1)
            - flags_as_felts[FLAG_PC_UPDATE_JUMP].clone()
            - flags_as_felts[FLAG_PC_UPDATE_JUMP_REL].clone()
            - flags_as_felts[FLAG_PC_UPDATE_JNZ].clone();
        // Assert pc_update = 0 / 1 / 2 / 4
        ab.constrain(
            const_expr!(1)
                - pc_update_regular.clone()
                - flags_as_felts[FLAG_PC_UPDATE_JUMP].clone()
                - flags_as_felts[FLAG_PC_UPDATE_JUMP_REL].clone()
                - flags_as_felts[FLAG_PC_UPDATE_JNZ].clone(),
        );
        flags_as_felts.push(pc_update_regular);

        // ap_update_regular = 1 iff FLAG_AP_UPDATE_ADD = 0 and FLAG_AP_UPDATE_ADD_1 = 0 and FLAG_OPCODE_CALL = 0
        let ap_update_regular = const_expr!(1)
            - flags_as_felts[FLAG_AP_UPDATE_ADD].clone()
            - flags_as_felts[FLAG_AP_UPDATE_ADD_1].clone()
            - flags_as_felts[FLAG_OPCODE_CALL].clone();
        // ab.deduce(&mut ap_update_regular);
        // Assert ap_update = 0 / 1 / 2 /4
        ab.constrain(
            const_expr!(1)
                - flags_as_felts[FLAG_AP_UPDATE_ADD].clone()
                - flags_as_felts[FLAG_AP_UPDATE_ADD_1].clone()
                - flags_as_felts[FLAG_OPCODE_CALL].clone()
                - ap_update_regular,
        );
        // flags_as_felts[FLAG_AP_UPDATE_REGULAR]=ap_update_regular;

        // fp_update_regular = 1 iff FLAG_OPCODE_CALL = 0 and FLAG_OPCODE_RET = 0
        let fp_update_regular = const_expr!(1)
            - flags_as_felts[FLAG_OPCODE_CALL].clone()
            - flags_as_felts[FLAG_OPCODE_RET].clone();
        // Assert that opcode = 0 / 1 / 2 /4
        ab.constrain(
            const_expr!(1)
                - fp_update_regular.clone()
                - flags_as_felts[FLAG_OPCODE_CALL].clone()
                - flags_as_felts[FLAG_OPCODE_RET].clone(),
        );
        flags_as_felts.push(fp_update_regular);

        // push instruction size
        flags_as_felts.push(const_expr!(1) + flags_as_felts[FLAG_OP1_IMM].clone());
        (flags_as_felts, offsets)
    }

    // Returns an Operands instance containing the values of the operands.
    fn eval_opernads(
        &self,
        ab: &mut AirBuilder,
        [pc, ap, fp]: CasmState,
        [offset0, offset1, offset2]: [GenericExprImpl<M31>; 3],
        flags: Vec<GenericExprImpl<Felt>>,
    ) -> [GenericExprImpl<Felt252>; 4] {
        // We read 252 bits since we don't know for what purpose is the reading
        let read_felt_252 = ReadSmallFelt252 {
            num_bits: FELT252_BITS_PER_WORD,
            memory: self.memory.clone(),
        };
        // read dst
        let dst_src = flags[FLAG_DST_BASE_FP].clone().clone() * fp.clone()
            + (const_expr!(1) - flags[FLAG_DST_BASE_FP].clone()) * ap.clone();
        let dst = ab.call(&read_felt_252, dst_src + offset0);

        // read op0
        let op0_src = flags[FLAG_OP0_BASE_FP].clone() * fp.clone()
            + (const_expr!(1) - flags[FLAG_OP0_BASE_FP].clone()) * ap.clone();
        let op0 = ab.call(&read_felt_252, op0_src + offset1);
        let op0_as_addr = op0.get_felt(0)
            + (op0.get_felt(1) * const_expr!(1 << FELT252_BITS_PER_WORD))
            + (op0.get_felt(2) * const_expr!(1 << (FELT252_BITS_PER_WORD * 2)));

        // read op1
        let op1_src = flags[FLAG_OP1_BASE_FP].clone() * fp.clone()
            + flags[FLAG_OP1_BASE_AP].clone() * ap.clone()
            + flags[FLAG_OP1_IMM].clone() * pc.clone()
            + flags[FLAG_OP1_BASE_OP0].clone() * op0_as_addr;
        let op1 = ab.call(&read_felt_252, op1_src + offset2);

        // Currently assumned res is M31
        // TODO - change to support felt252 operaation
        let res =ab.assign(&mut (flags[FLAG_RES_OP1].clone() * op1.get_felt(0)
            + flags[FLAG_RES_MUL].clone() * (op0.get_felt(0) * op1.get_felt(0))
            + flags[FLAG_RES_ADD].clone() * (op0.get_felt(0) + op1.get_felt(0))));


        [dst, op0, op1, Felt252Expr::from(vec![res])]
    }

    fn update_registers(
        &self,
        ab: &mut AirBuilder,
        [pc, ap, fp]: CasmState,
        flags: Vec<GenericExprImpl<Felt>>,
        [dst, op1, res]: [GenericExprImpl<Felt252>; 3],
    ) -> CasmState {
        let op1_as_addr = op1.get_felt(0)
            + (op1.get_felt(1) * const_expr!(1 << FELT252_BITS_PER_WORD))
            + (op1.get_felt(2) * const_expr!(1 << (FELT252_BITS_PER_WORD * 2)));
        let res_as_addr = res.get_felt(0)
            + (res.get_felt(1) * const_expr!(1 << FELT252_BITS_PER_WORD))
            + (res.get_felt(2) * const_expr!(1 << (FELT252_BITS_PER_WORD * 2)));
        let dst_as_addr = dst.get_felt(0)
            + (dst.get_felt(1) * const_expr!(1 << FELT252_BITS_PER_WORD))
            + (dst.get_felt(2) * const_expr!(1 << (FELT252_BITS_PER_WORD * 2)));

        // handle jnz
        let dst_sum = dst
            .as_felts()
            .into_iter()
            .fold(const_expr!(0), |acc, x| acc + x);
        // If dst == 0 : dst_is_zero = 1 else dst_is_zero = 0
        let dst_is_zero =
            ab.assign(&mut (const_expr!(1) - (const_expr!(1) / dst_sum.clone()) * dst_sum));
        let dst_sum_squares = dst
            .as_felts()
            .into_iter()
            .enumerate()
            .map(|(i, x)| {
                if P_FELTS[i] == 0 {
                    x
                } else {
                    let x = ab.let_(x - const_expr!(P_FELTS[i]));
                    x.clone() * x
                }
            })
            .fold(const_expr!(0), |acc, z| acc + z);
        // If dst == p : dst_is_p = 1 else dst_is_p = 0
        let dst_is_p = ab.assign(
            &mut (const_expr!(1) - (const_expr!(1) / dst_sum_squares.clone()) * dst_sum_squares),
        );
        // If dst == 0 or dst == p : next_pc = pc + instruction_size else next_pc = pc + op1
        let npc_jnz = ab.assign(
            &mut ((dst_is_zero.clone() + dst_is_p.clone())
                * (pc.clone() + flags[INSTRUCTION_SIZE].clone())
                + (const_expr!(1) - dst_is_zero.clone() - dst_is_p.clone())
                    * (pc.clone() + op1_as_addr.clone())),
        );

        // update fp
        let next_pc = flags[FLAG_PC_UPDATE_REGULAR].clone()
            * (pc.clone() + flags[INSTRUCTION_SIZE].clone())
            + flags[FLAG_PC_UPDATE_JUMP].clone() * res_as_addr.clone()
            + flags[FLAG_PC_UPDATE_JUMP_REL].clone() * (pc.clone() + res_as_addr.clone())
            + flags[FLAG_PC_UPDATE_JNZ].clone() * npc_jnz;

        // update ap
        let next_ap = ap.clone()
            + flags[FLAG_AP_UPDATE_ADD].clone() * res_as_addr
            + flags[FLAG_AP_UPDATE_ADD_1].clone() * const_expr!(1)
            + flags[FLAG_OPCODE_CALL].clone() * const_expr!(2);

        // update fp
        let next_fp = flags[FLAG_FP_UPDATE_REGULAR].clone() * fp.clone()
            + flags[FLAG_OPCODE_RET].clone() * dst_as_addr
            + flags[FLAG_OPCODE_CALL].clone() * (ap.clone() + const_expr!(2));

        [next_pc, next_ap, next_fp]
    }
}

impl AirFn for GenericOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        let (flags_as_felts, offsets) = self.decode_instruction(ab, pc.clone());
        let [dst, op0, op1, res] = self.eval_opernads(
            ab,
            [pc.clone(), ap.clone(), fp.clone()],
            offsets.clone(),
            flags_as_felts.clone(),
        );

        // handle assert_eq
        for (dest_felt, res_felt) in dst.as_felts().into_iter().zip(res.as_felts().into_iter()) {
            ab.constrain(flags_as_felts[FLAG_OPCODE_ASSERT_EQ].clone() * (res_felt - dest_felt));
        }

        //handle ret
        // offset0 = 2
        ab.constrain(
            flags_as_felts[FLAG_OPCODE_RET].clone() * (offsets[0].clone() + const_expr!(2)),
        );
        // offset2 = 1
        ab.constrain(
            flags_as_felts[FLAG_OPCODE_RET].clone() * (offsets[2].clone() + const_expr!(1)),
        );
        // Assert that FLAG_PC_UPDATE_JUMP = FLAG_DST_BASE_FP = FLAG_OP1_BASE_FP =FLAG_RES_OP1 = 1
        ab.constrain(
            flags_as_felts[FLAG_OPCODE_RET].clone()
                * (const_expr!(4)
                    - flags_as_felts[FLAG_PC_UPDATE_JUMP].clone()
                    - flags_as_felts[FLAG_DST_BASE_FP].clone()
                    - flags_as_felts[FLAG_OP1_BASE_FP].clone()
                    - flags_as_felts[FLAG_RES_OP1].clone()),
        );
        //handle call
        // ofsset0 = 0
        ab.constrain(flags_as_felts[FLAG_OPCODE_CALL].clone() * offsets[0].clone());
        // offset1 = 1
        ab.constrain(
            flags_as_felts[FLAG_OPCODE_CALL].clone() * (const_expr!(1) - offsets[1].clone()),
        );
        // Assert that FLAG_OP0_BASE_FP = FLAG_DST_BASE_FP = 0
        ab.constrain(
            flags_as_felts[FLAG_OPCODE_CALL].clone()
                * (flags_as_felts[FLAG_OP0_BASE_FP].clone()
                    + flags_as_felts[FLAG_DST_BASE_FP].clone()),
        );
        // push fp
        ab.constrain(flags_as_felts[FLAG_OPCODE_CALL].clone() * (dst.get_felt(0) - fp.clone()));

        // push pc
        ab.constrain(flags_as_felts[FLAG_OPCODE_CALL].clone() * (op0.get_felt(0) - (pc.clone() + flags_as_felts[INSTRUCTION_SIZE].clone())));

        self.update_registers(ab, [pc, ap, fp], flags_as_felts, [dst, op1, res])
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

impl MemoryAirFn for GenericOpcode {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
