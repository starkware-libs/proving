use clap::{Parser, Subcommand};

use air_infra::airs::examples::bit_unpacking::create_bit_unpacking_json;
use air_infra::airs::examples::fibonacci::create_fibonacci_json;
use air_infra::airs::uint32_utils::create_add32_json;
use air_infra::utils::create_opcodes_json::*;
use air_infra::utils::fn_sizes::*;

#[derive(Subcommand, Debug)]
enum MainCommands {
    FnSizes,
    WriteJsonExample { fn_name: String }, // Usage: cargo run write-json-example <function_name>
    WriteJsonCasmOpcode(CasmOpcodeCommand), // Usage: cargo run write-json-casm-opcode <opcode_name> --<flags set to true>
}

#[derive(Parser, Debug)]
struct MainArgs {
    #[command(subcommand)]
    command: MainCommands,
}

pub fn write_json_example(fn_name: String) {
    if fn_name == "fib" {
        println!("Creating a json file for fibonacci");
        create_fibonacci_json();
    }

    if fn_name == "add32" {
        println!("Creating a json file for add32");
        create_add32_json();
    }

    if fn_name == "bit_unpack" {
        println!("Creating a json file for bit unpacking");
        create_bit_unpacking_json();
    }
}

pub fn main() {
    let args = MainArgs::parse();

    match args.command {
        MainCommands::FnSizes => print_fn_sizes(),
        MainCommands::WriteJsonExample { fn_name } => write_json_example(fn_name),
        MainCommands::WriteJsonCasmOpcode(opcode) => match opcode.command {
            CasmOpcodeSubCommand::AssertEqual(assert_eq_opcode_args) => {
                create_assert_equal_opcode_json(assert_eq_opcode_args)
            }
            CasmOpcodeSubCommand::Call(call_opcode_args) => {
                create_call_opcode_json(call_opcode_args)
            }
            CasmOpcodeSubCommand::Jump(jump_opcode_args) => {
                create_jump_opcode_json(jump_opcode_args)
            }
            CasmOpcodeSubCommand::Ret(_) => create_ret_opcode_json(),
        },
    };
}
