use clap::Args;
use clap::Subcommand;

use crate::airs::casm::opcodes::assert_eq_opcode::*;
use crate::airs::casm::opcodes::call_opcode::*;
use crate::airs::casm::opcodes::jump_opcode::*;
use crate::airs::casm::opcodes::ret_opcode::*;
use crate::airs::examples::bit_unpacking::bit_unpack::*;
use crate::airs::examples::fibonacci::fib::*;
use crate::airs::memory::felt252_id_memory::*;
use crate::airs::uint32_utils::add32::*;
use crate::core::air_fn_registry::*;

#[derive(Args, Debug)]
pub struct WriteJsonCommand {
    #[clap(subcommand)]
    pub name: WriteJsonSubCommand,
}

#[derive(Subcommand, Debug)]
pub enum WriteJsonSubCommand {
    Fib(FibonachiArgs),
    Add32,
    BitUnpack,
    Ret,
    AssertEqual(AssertEqOpcodeArgs),
    Call(CallOpcodeArgs),
    Jump(JumpOpcodeArgs),
}

#[derive(Debug)]
pub enum AirFnArgs {
    AssertEqOpcodeArgs(AssertEqOpcodeArgs),
    CallOpcodeArgs(CallOpcodeArgs),
    JumpOpcodeArgs(JumpOpcodeArgs),
    FibonachiArgs(FibonachiArgs),
    Add32Args(),
    BitUnpackArgs(),
    RetOpcodeArgs(),
}

#[derive(Debug, Args)]
pub struct FibonachiArgs {
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

pub fn create_air_fn_json(arguments: AirFnArgs) -> AirFnRegistry {
    match arguments {
        AirFnArgs::AssertEqOpcodeArgs(arguments) => AirFnRegistry::new(&AssertEqOpcode {
            is_imm: arguments.is_imm,
            is_double_deref: arguments.is_double_deref,
            memory: Felt252IdMemory::default(),
        }),
        AirFnArgs::CallOpcodeArgs(arguments) => AirFnRegistry::new(&CallOpcode {
            is_rel: arguments.is_rel,
            op1_base_fp: arguments.flag_op1_base_fp,
            memory: Felt252IdMemory::default(),
        }),
        AirFnArgs::JumpOpcodeArgs(arguments) => AirFnRegistry::new(&JumpOpcode {
            is_rel: arguments.is_rel,
            is_imm: arguments.is_imm,
            is_double_deref: arguments.is_double_deref,
            memory: Felt252IdMemory::default(),
        }),
        AirFnArgs::FibonachiArgs(arguments) => AirFnRegistry::new(&Fib {
            claim_index: arguments.claim_index,
        }),
        AirFnArgs::Add32Args() => AirFnRegistry::new(&Add32 {}),
        AirFnArgs::BitUnpackArgs() => AirFnRegistry::new(&BitUnpack::<4> {}),
        AirFnArgs::RetOpcodeArgs() => AirFnRegistry::new(&RetOpcode {
            memory: Felt252IdMemory::default(),
        }),
    }
}
