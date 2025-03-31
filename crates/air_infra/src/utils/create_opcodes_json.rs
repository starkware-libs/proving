use clap::{Args, Subcommand};

use crate::airs::casm::opcodes::assert_eq_opcode::*;
use crate::airs::casm::opcodes::call_opcode::*;
use crate::airs::casm::opcodes::jump_opcode::*;
use crate::airs::casm::opcodes::ret_opcode::*;
use crate::airs::examples::bit_unpacking::bit_unpack::*;
use crate::airs::examples::fibonacci::fib::*;
use crate::core::air_fn_registry::*;
use crate::core::felt252_id_memory::memory::*;

#[derive(Args, Debug)]
pub struct WriteJsonCommand {
    #[clap(subcommand)]
    pub args: AirFnArgs,
}

#[derive(Subcommand, Debug)]
pub enum AirFnArgs {
    Fib(FibArgs),
    BitUnpack,
    Ret,
    AssertEq(AssertEqOpcodeArgs),
    Call(CallOpcodeArgs),
    Jump(JumpOpcodeArgs),
}

#[derive(Debug, Args)]
pub struct FibArgs {
    #[clap(long)]
    claim_index: usize,
}

#[derive(Debug, Args)]
pub struct AssertEqOpcodeArgs {
    #[clap(long)]
    double_deref: bool,
    #[clap(long)]
    imm: bool,
}

#[derive(Debug, Args)]
pub struct CallOpcodeArgs {
    #[clap(long)]
    rel: bool,
    #[clap(long)]
    flag_op1_base_fp: bool,
}

#[derive(Debug, Args)]
pub struct JumpOpcodeArgs {
    #[clap(long)]
    rel: bool,
    #[clap(long)]
    imm: bool,
    #[clap(long)]
    double_deref: bool,
}

pub fn create_air_fn_registry(arguments: AirFnArgs) -> AirFnRegistry {
    let mut registry = AirFnRegistry::new_empty();
    match arguments {
        AirFnArgs::AssertEq(arguments) => registry.add_entry(&AssertEqOpcode {
            imm: arguments.imm,
            double_deref: arguments.double_deref,
            memory: Felt252IdMemory::default(),
        }),
        AirFnArgs::Call(arguments) => registry.add_entry(&CallOpcode {
            rel: arguments.rel,
            op1_base_fp: arguments.flag_op1_base_fp,
            memory: Felt252IdMemory::default(),
        }),
        AirFnArgs::Jump(arguments) => registry.add_entry(&JumpOpcode {
            rel: arguments.rel,
            imm: arguments.imm,
            double_deref: arguments.double_deref,
            memory: Felt252IdMemory::default(),
        }),
        AirFnArgs::Fib(arguments) => registry.add_entry(&Fib {
            claim_index: arguments.claim_index,
        }),
        AirFnArgs::BitUnpack => registry.add_entry(&BitUnpack::<4>::new()),
        AirFnArgs::Ret => registry.add_entry(&RetOpcode::default()),
    };
    registry
}
