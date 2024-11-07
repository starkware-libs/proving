use clap::{Args, Subcommand};

use crate::airs::casm::opcodes::assert_eq_opcode::*;
use crate::airs::casm::opcodes::call_opcode::*;
use crate::airs::casm::opcodes::jump_opcode::*;
use crate::airs::casm::opcodes::ret_opcode::*;
use crate::airs::examples::bit_unpacking::bit_unpack::*;
use crate::airs::examples::fibonacci::fib::*;
use crate::airs::felt252_id_memory::memory::*;
use crate::airs::uint32_utils::add32::*;
use crate::core::air_fn_registry::*;

#[derive(Args, Debug)]
pub struct WriteJsonCommand {
    #[clap(subcommand)]
    pub args: AirFnArgs,
}

#[derive(Subcommand, Debug)]
pub enum AirFnArgs {
    Fib(FibArgs),
    Add32,
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
    is_double_deref: bool,
    #[clap(long)]
    is_imm: bool,
}

#[derive(Debug, Args)]
pub struct CallOpcodeArgs {
    #[clap(long)]
    is_rel: bool,
    #[clap(long)]
    flag_op1_base_fp: bool,
}

#[derive(Debug, Args)]
pub struct JumpOpcodeArgs {
    #[clap(long)]
    is_rel: bool,
    #[clap(long)]
    is_imm: bool,
    #[clap(long)]
    is_double_deref: bool,
}

pub fn create_air_fn_registry(arguments: AirFnArgs) -> AirFnRegistry {
    let mut registry = AirFnRegistry::new_empty();
    match arguments {
        AirFnArgs::AssertEq(arguments) => registry.add_entry(&AssertEqOpcode {
            is_imm: arguments.is_imm,
            is_double_deref: arguments.is_double_deref,
            memory: Felt252IdMemory::default(),
        }),
        AirFnArgs::Call(arguments) => registry.add_entry(&CallOpcode {
            is_rel: arguments.is_rel,
            op1_base_fp: arguments.flag_op1_base_fp,
            memory: Felt252IdMemory::default(),
        }),
        AirFnArgs::Jump(arguments) => registry.add_entry(&JumpOpcode {
            is_rel: arguments.is_rel,
            is_imm: arguments.is_imm,
            is_double_deref: arguments.is_double_deref,
            memory: Felt252IdMemory::default(),
        }),
        AirFnArgs::Fib(arguments) => registry.add_entry(&Fib {
            claim_index: arguments.claim_index,
        }),
        AirFnArgs::Add32 => registry.add_entry(&Add32 {}),
        AirFnArgs::BitUnpack => registry.add_entry(&BitUnpack::<4> {}),
        AirFnArgs::Ret => registry.add_entry(&RetOpcode::default()),
    };
    registry
}
