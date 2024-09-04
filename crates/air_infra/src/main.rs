use clap::{Parser, Subcommand};

use air_infra::airs::examples::bit_unpacking::create_bit_unpacking_json;
use air_infra::airs::examples::fibonacci::create_fibonacci_json;
use air_infra::airs::uint32_utils::create_add32_json;
use air_infra::utils::create_opcodes_json::*;
use air_infra::utils::fn_sizes::*;

#[derive(Subcommand, Debug)]
enum MainCommands {
    FnSizes,
    WriteJson(WriteJsonCommand), // Usage: cargo run write-json opcode/example_name --<flags set to true>
}

#[derive(Parser, Debug)]
struct MainArgs {
    #[command(subcommand)]
    command: MainCommands,
}

pub fn main() {
    let args = MainArgs::parse();

    match args.command {
        MainCommands::FnSizes => print_fn_sizes(),
        MainCommands::WriteJson(command) => match command.name {
            WriteJsonSubCommand::Fib => {
                println!("Creating a json file for fibonacci");
                create_fibonacci_json().dump_to_file(None, None);
            }
            WriteJsonSubCommand::Add32 => {
                println!("Creating a json file for add32");
                create_add32_json().dump_to_file(None, None);
            }
            WriteJsonSubCommand::BitUnpack => {
                println!("Creating a json file for bit unpacking");
                create_bit_unpacking_json().dump_to_file(None, None);
            }
            WriteJsonSubCommand::Ret => create_ret_opcode_json().dump_to_file(None, None),
            WriteJsonSubCommand::AssertEqual(assert_eq_opcode_args) => {
                create_assert_equal_opcode_json(assert_eq_opcode_args).dump_to_file(None, None)
            }
            WriteJsonSubCommand::Call(call_opcode_args) => {
                create_call_opcode_json(call_opcode_args).dump_to_file(None, None)
            }
            WriteJsonSubCommand::Jump(jump_opcode_args) => {
                create_jump_opcode_json(jump_opcode_args).dump_to_file(None, None)
            }
        },
    }
}
