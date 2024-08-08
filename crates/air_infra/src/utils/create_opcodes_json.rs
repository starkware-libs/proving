use clap::Args;
use clap::Subcommand;

use crate::airs::casm::assert_eq_opcode::*;
use crate::airs::casm::call_opcode::*;
use crate::airs::casm::jump_opcode::*;
use crate::airs::casm::ret_opcode::*;
use crate::core::air_fn_registry::*;
use crate::core::memory::Memory;

#[derive(Args, Debug)]
pub struct CasmOpcodeCommand {
    #[clap(subcommand)]
    pub command: CasmOpcodeSubCommand,
}

#[derive(Subcommand, Debug)]
pub enum CasmOpcodeSubCommand {
    AssertEqual(AssertEqOpcodeArgs),
    Call(CallOpcodeArgs),
    Jump(JumpOpcodeArgs),
    Ret(RetOpcodeArgs),
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

#[derive(Debug, Args)]
pub struct RetOpcodeArgs {}

pub fn create_assert_equal_opcode_json(arguments: AssertEqOpcodeArgs) {
    println!(
        "Creating a json file for assert_equal_opcode with arguments: {:?}",
        arguments
    );
    let registry = AirFnRegistry::new(&AssertEqOpcode {
        is_double_deref: arguments.is_double_deref,
        is_immediate: arguments.is_immediate,
        memory: Memory::default(),
    });
    registry.dump_to_file("airs/casm/assert_equal_opcode.json");
}

pub fn create_call_opcode_json(arguments: CallOpcodeArgs) {
    println!(
        "Creating a json file for call_opcode with arguments: {:?}",
        arguments
    );
    let registry = AirFnRegistry::new(&CallOpcode {
        is_rel: arguments.is_rel,
        flag_op1_base_fp: arguments.flag_op1_base_fp,
        memory: Memory::default(),
    });
    registry.dump_to_file("airs/casm/call_opcode.json");
}

pub fn create_jump_opcode_json(arguments: JumpOpcodeArgs) {
    println!(
        "Creating a json file for jump_opcode with arguments: {:?}",
        arguments
    );
    let registry = AirFnRegistry::new(&JumpOpcode {
        is_rel: arguments.is_rel,
        flag_op1_base_fp: arguments.flag_op1_base_fp,
        flag_ap_update_add_1: arguments.flag_ap_update_add_1,
        memory: Memory::default(),
    });
    registry.dump_to_file("airs/casm/jump_opcode.json");
}

pub fn create_ret_opcode_json() {
    println!("Creating a json file for ret_opcode");
    let registry = AirFnRegistry::new(&RetOpcode {
        memory: Memory::default(),
    });
    registry.dump_to_file("airs/casm/ret_opcode.json");
}
