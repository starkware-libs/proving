use clap::Args;
use clap::Subcommand;

use crate::airs::casm::opcodes::assert_eq_opcode::*;
use crate::airs::casm::opcodes::call_opcode::*;
use crate::airs::casm::opcodes::jump_opcode::*;
use crate::airs::casm::opcodes::ret_opcode::*;
use crate::core::air_fn_registry::*;
use crate::core::memory::Memory;

#[derive(Args, Debug)]
pub struct WriteJsonCommand {
    #[clap(subcommand)]
    pub name: WriteJsonSubCommand,
    #[arg(short, long)]
    pub output: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum WriteJsonSubCommand {
    Fib,
    Add32,
    BitUnpack,
    Ret,
    AssertEqual(AssertEqOpcodeArgs),
    Call(CallOpcodeArgs),
    Jump(JumpOpcodeArgs),
}

#[derive(Debug, Args)]
pub struct AssertEqOpcodeArgs {
    #[clap(long)]
    is_double_deref: bool,
    #[clap(long)]
    is_immediate: bool,
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
    flag_op1_base_fp: bool,
    #[clap(long)]
    flag_ap_update_add_1: bool,
}

pub fn create_assert_equal_opcode_json(arguments: AssertEqOpcodeArgs) -> AirFnRegistry {
    println!(
        "Creating a json file for assert_equal_opcode with arguments: {:?}",
        arguments
    );
    AirFnRegistry::new(&AssertEqOpcode {
        is_double_deref: arguments.is_double_deref,
        is_immediate: arguments.is_immediate,
        memory: Memory::default(),
    })
}

pub fn create_call_opcode_json(arguments: CallOpcodeArgs) -> AirFnRegistry {
    println!(
        "Creating a json file for call_opcode with arguments: {:?}",
        arguments
    );
    AirFnRegistry::new(&CallOpcode {
        is_rel: arguments.is_rel,
        flag_op1_base_fp: arguments.flag_op1_base_fp,
        memory: Memory::default(),
    })
}

pub fn create_jump_opcode_json(arguments: JumpOpcodeArgs) -> AirFnRegistry {
    println!(
        "Creating a json file for jump_opcode with arguments: {:?}",
        arguments
    );
    AirFnRegistry::new(&JumpOpcode {
        is_rel: arguments.is_rel,
        flag_op1_base_fp: arguments.flag_op1_base_fp,
        flag_ap_update_add_1: arguments.flag_ap_update_add_1,
        memory: Memory::default(),
    })
}

pub fn create_ret_opcode_json() -> AirFnRegistry {
    println!("Creating a json file for ret_opcode");
    AirFnRegistry::new(&RetOpcode {
        memory: Memory::default(),
    })
}
