use crate::airs::memory::felt252_id_memory_read_small::ReadSmall;
use crate::core::prover_types::Felt252;
use crate::core::prover_types::FELT252_BITS_PER_WORD;
use crate::core::air_fn::*;
use crate::core::expressions::expr::GenericExprImpl;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::Felt;
use crate::airs::casm::read_addr::*;
use crate::core::expressions::bool_expr::BoolExpr;
use stwo_prover::core::fields::m31::M31;
use crate::core::variables::AirVar;
use crate::core::prover_types::P_FELTS;

use super::check_instruction::*;
use super::common::*;
use super::read_small_felt252::ReadSmallFelt252;

// Macros
use crate::const_expr;

pub const FLAG_OP1_BASE_OP0: usize = 15;
pub const FLAG_RES_OP1: usize = 16;
pub const FLAG_PC_UPDATE_REGULAR: usize = 17;
pub const FLAG_AP_UPDATE_REGULAR: usize = 18;
pub const FLAG_FP_UPDATE_REGULAR: usize = 19;
pub const INSTRUCTION_SIZE: usize = 20;
pub const NUM_LIMBS_PER_ADDR: usize = 3;

/// Implements a generic Cairo0 instructions.
/// All the flags are read from the trace.

#[derive(Clone, Debug)]
pub struct GenericOpcode {
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl GenericOpcode {
    fn decode_instruction(&self, ab: &mut AirBuilder, flags: [BoolExpr; 15]) -> Vec<GenericExprImpl<Felt>> {

        let mut flags_as_felts = flags.iter().map(|f| f.as_felt()).collect::<Vec<_>>();

        // Add constraints on the flags
        // op1_base_op0 = 1 iff FLAG_OP1_IMM = 0 and FLAG_OP1_BASE_FP = 0 and FLAG_OP1_BASE_AP = 0
        let mut op1_base_op0 = ab.let_for_deduction((const_expr!(1)-flags_as_felts[FLAG_OP1_IMM])*(const_expr!(1)-flags_as_felts[FLAG_OP1_BASE_FP])*(const_expr!(1)-flags_as_felts[FLAG_OP1_BASE_AP]));
        ab.deduce(&mut op1_base_op0);
        // Assert op1 = 0 / 1 / 2 / 4
        ab.constrain(const_expr!(1) - op1_base_op0.clone() - flags_as_felts[FLAG_OP1_IMM] - flags_as_felts[FLAG_OP1_BASE_FP] - flags_as_felts[FLAG_OP1_BASE_AP]);
        flags_as_felts.push(op1_base_op0);

        // res_op1 = 1 iff FLAG_RES_ADD = 0 and FLAG_RES_MUL = 0
        let mut res_op1 = ab.let_for_deduction((const_expr!(1)- flags_as_felts[FLAG_RES_ADD])*(const_expr!(1)- flags_as_felts[FLAG_RES_MUL]));
        ab.deduce(&mut res_op1);
        // Assert res_logic = 0 / 1 / 2
        ab.constrain(const_expr!(1) - res_op1.clone() - flags_as_felts[FLAG_RES_ADD] - flags_as_felts[FLAG_RES_MUL]);
        flags_as_felts.push(res_op1);

        // pc_update_regular = 1 iff FLAG_PC_UPDATE_JUMP = 0 and FLAG_PC_UPDATE_JUMP_REL = 0 and FLAG_PC_UPDATE_JNZ = 0
        let mut pc_update_regular = ab.let_for_deduction((const_expr!(1)-flags_as_felts[FLAG_PC_UPDATE_JUMP])*(const_expr!(1)-flags_as_felts[FLAG_PC_UPDATE_JUMP_REL])*(const_expr!(1)-flags_as_felts[FLAG_PC_UPDATE_JNZ]));
        ab.deduce(&mut pc_update_regular);
        // Assert pc_update = 0 / 1 / 2 / 4
        ab.constrain(const_expr!(1)- pc_update_regular.clone() - flags_as_felts[FLAG_PC_UPDATE_JUMP] - flags_as_felts[FLAG_PC_UPDATE_JUMP_REL] - flags_as_felts[FLAG_PC_UPDATE_JNZ]);
        flags_as_felts.push(pc_update_regular);

        // ap_update_regular = 1 iff FLAG_AP_UPDATE_ADD = 0 and FLAG_AP_UPDATE_ADD_1 = 0 and FLAG_OPCODE_CALL = 0
        let mut ap_update_regular = ab.let_for_deduction((const_expr!(1)-flags_as_felts[FLAG_AP_UPDATE_ADD])*(const_expr!(1)-flags_as_felts[FLAG_AP_UPDATE_ADD_1])*(const_expr!(1)-flags_as_felts[FLAG_OPCODE_CALL]));
        ab.deduce(&mut ap_update_regular);
        // Assert ap_update = 0 / 1 / 2
        ab.constrain(const_expr!(1)-ap_update_regular.clone()-flags_as_felts[FLAG_AP_UPDATE_ADD]-flags_as_felts[FLAG_AP_UPDATE_ADD_1]-flags_as_felts[FLAG_OPCODE_CALL]);
        flags_as_felts.push(ap_update_regular);

        // fp_update_regular = 1 iff FLAG_OPCODE_CALL = 0 and FLAG_OPCODE_RET = 0 and FLAG_OPCODE_ASSERT_EQ = 0
        let mut fp_update_regular = ab.let_for_deduction((const_expr!(1)-flags_as_felts[FLAG_OPCODE_CALL])*(const_expr!(1)-flags_as_felts[FLAG_OPCODE_RET])*(const_expr!(1)-flags_as_felts[FLAG_OPCODE_ASSERT_EQ]));
       // Assert that opcode = 0 / 1 / 2 /4
        ab.constrain(const_expr!(1)-fp_update_regular.clone()-flags_as_felts[FLAG_OPCODE_CALL]-flags_as_felts[FLAG_OPCODE_RET]-flags_as_felts[FLAG_OPCODE_ASSERT_EQ]);
        flags_as_felts.push(fp_update_regular);

        // push instruction size
        flags_as_felts.push(const_expr!(1) + flags_as_felts[FLAG_OP1_IMM]);
        flags_as_felts
    }

    fn eval_opernads(self, ab: &mut AirBuilder, [pc, ap, fp]: CasmState, [offset0, offset1, offset2]: [GenericExprImpl<M31>; 3], flags: Vec<GenericExprImpl<Felt>>) ->  [GenericExprImpl<Felt252>; 4]{
        // read dst
        let dst_src = flags[FLAG_DST_BASE_FP]*fp.clone() + (const_expr!(1)-flags[FLAG_DST_BASE_FP])*ap.clone();
        let read_felt_252 = ReadSmallFelt252{
            num_bits: FELT252_BITS_PER_WORD,
            memory: self.memory.clone(),
        };
        let dst = ab.call(&read_felt_252, dst_src + offset0);

        // read op0
        let op0_src = flags[FLAG_OP0_BASE_FP]*fp.clone() + (const_expr!(1)-flags[FLAG_OP0_BASE_FP])*ap.clone();
        let op0 = ab.call(&read_felt_252, op0_src + offset1);
        let op0_as_addr = op0.get_felt(0)
        + (op0.get_felt(1) * const_expr!(1 << FELT252_BITS_PER_WORD))
        + (op0.get_felt(2) * const_expr!(1 << (FELT252_BITS_PER_WORD * 2)));

        // read op1
        let op1_src = flags[FLAG_OP1_BASE_FP]*fp.clone() + flags[FLAG_OP1_BASE_AP]*ap.clone() + flags[FLAG_OP1_IMM]*pc.clone() + flags[FLAG_OP1_BASE_OP0]*op0_as_addr;
        let op1 = ab.call(&read_felt_252, op1_src + offset2);

        let res =  Felt252Expr::from(flags[FLAG_RES_OP1])*op1 + Felt252Expr::from(flags[FLAG_RES_MUL])*(op0*op1) + Felt252Expr::from(flags[FLAG_RES_ADD])*(op0+op1);

        [dst, op0, op1, res]
    }

    fn update_registers(&self, ab: &mut AirBuilder, [pc, ap, fp]: CasmState, flags: Vec<GenericExprImpl<Felt>>, [offset0, offset1, offset2]: [GenericExprImpl<M31>; 3], [dst, op0, op1, res]: [GenericExprImpl<Felt252>; 4]) -> CasmState {
        let op1_as_addr = op1.get_felt(0)
        + (op1.get_felt(1) * const_expr!(1 << FELT252_BITS_PER_WORD))
        + (op1.get_felt(2) * const_expr!(1 << (FELT252_BITS_PER_WORD * 2)));
        let res_as_addr = res.get_felt(0)
        + (res.get_felt(1) * const_expr!(1 << FELT252_BITS_PER_WORD))
        + (res.get_felt(2) * const_expr!(1 << (FELT252_BITS_PER_WORD * 2)));
        let dst_as_addr = dst.get_felt(0) + (dst.get_felt(1) * const_expr!(1 << FELT252_BITS_PER_WORD)) + (dst.get_felt(2) * const_expr!(1 << (FELT252_BITS_PER_WORD * 2)));
        // handle jnz
        let dst_sum = dst.as_felts().into_iter().fold(const_expr!(0), |acc, x| acc + x);
        let dst_sum_inv = ab.deduce(&mut (const_expr!(1) / dst_sum.clone()));
        let dst_sum_squares = dst.as_felts()
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
        let next_pc = ab.deduce()
        dst_sum_squares_inv = ab.deduce(&mut (const_expr!(1) / dst_sum_squares.clone()));
        let pc_jnz = (const_expr!(1)-dst_is_zero)*(pc.clone() + op1_as_addr) + dst.is_zero()*(pc.clone() + flags[INSTRUCTION_SIZE]);
        let npc_jnz = ab.let_for_deduction((const_expr!(2)-dst_sum * dst_sum_inv-dst_sum_squares-dst_sum_squares_inv*dst_sum_squares)*);
        ab.constrain(dst_sum * dst_sum_inv - const_expr!(1));



        let next_pc = flags[FLAG_PC_UPDATE_REGULAR]*(pc.clone() + flags[INSTRUCTION_SIZE]) + flags[FLAG_PC_UPDATE_JUMP]*res_as_addr + flags[FLAG_PC_UPDATE_JUMP_REL]*(pc.clone() + res_as_addr) + flags[FLAG_PC_UPDATE_JNZ]*pc_jnz;

        // update ap
        let next_ap = flags[FLAG_AP_UPDATE_REGULAR]*(ap.clone()) + flags[FLAG_AP_UPDATE_ADD]*(ap.clone()+res_as_addr) + flags[FLAG_AP_UPDATE_ADD_1]*(ap.clone() + const_expr!(1)) + flags[FLAG_OPCODE_CALL]*(ap.clone()+const_expr!(2));

        // update fp
        let next_fp = flags[FLAG_FP_UPDATE_REGULAR]*(fp.clone()) + flags[FLAG_OPCODE_RET]*dst_as_addr + flags[FLAG_OPCODE_ASSERT_EQ]*fp.clone() + flags[FLAG_OPCODE_CALL]*(ap.clone()+const_expr!(2));

        [next_pc, next_ap, next_fp]

    }
}

impl AirFn for GenericOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        // Read the opcdode
        let (offsets, flags) = ab.call(
            &CheckInstruction {
                const_offsets: [None, None, None],
                const_flags: Default::default(),
                memory: self.memory.clone(),
            },
            pc.clone(),
        );
        let flags_as_felts = self.decode_instruction(ab, flags);
        let [dst, op0, op1, res] = self.eval_opernads(ab, [pc, ap, fp], offsets, flags_as_felts);

        // handle assert_eq
        for (dest_felt, res_felt) in dst.as_felts().into_iter().zip(res.as_felts().into_iter()) {
            ab.constrain(flags_as_felts[FLAG_OPCODE_ASSERT_EQ]*(res_felt - dest_felt));
        }

        //handle ret
        ab.constrain(flags_as_felts[FLAG_OPCODE_RET]*(offsets[0] + const_expr!(2)));
        ab.constrain(flags_as_felts[FLAG_OPCODE_RET]*(offsets[2] + const_expr!(1)));
        ab.constrain(flags_as_felts[FLAG_OPCODE_RET]*(const_expr!(4)-flags_as_felts[FLAG_PC_UPDATE_JUMP]-flags_as_felts[FLAG_DST_BASE_FP]-flags_as_felts[FLAG_OP1_BASE_FP]-flags_as_felts[FLAG_RES_OP1]));

        //handle call
        ab.constrain(flags_as_felts[FLAG_OPCODE_CALL]*(offsets[1] - pc.clone() - flags_as_felts[INSTRUCTION_SIZE])*(offsets[0] - fp.clone()));
        ab.constrain(flags_as_felts[FLAG_OPCODE_CALL]*offsets[0]);
        ab.constrain(flags_as_felts[FLAG_OPCODE_CALL]*(const_expr!(1)-offsets[1]));
        ab.constrain(flags_as_felts[FLAG_OPCODE_CALL]*(flags_as_felts[FLAG_OP0_BASE_FP] + flags_as_felts[FLAG_DST_BASE_FP]));
        for (dest_felt, fp_felt) in dst.as_felts()[..NUM_LIMBS_PER_ADDR].into_iter().zip(fp.as_felts()[..NUM_LIMBS_PER_ADDR].into_iter()) {

            ab.constrain(flags_as_felts[FLAG_OPCODE_CALL]*(*dest_felt - *fp_felt));
        }
        for (op0_felt, next_pc_felt) in op0.as_felts()[..NUM_LIMBS_PER_ADDR].into_iter().zip((pc + flags_as_felts[INSTRUCTION_SIZE]).as_felts()[..NUM_LIMBS_PER_ADDR].into_iter()) {
            ab.constrain(flags_as_felts[FLAG_OPCODE_CALL]*(*op0_felt - *next_pc_felt));
        }
        //handle jump
        self.update_registers(ab, [pc, ap, fp], flags_as_felts, offsets, [dst, op0, op1, res])
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