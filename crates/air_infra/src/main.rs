use air_infra::core::utils::dump_to_file;
use clap::{Parser, Subcommand};

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
        MainCommands::WriteJson(command) => {
            let air_fn_args = match command.name {
                WriteJsonSubCommand::Fib(fib_args) => AirFnArgs::FibonachiArgs(fib_args),
                WriteJsonSubCommand::Add32 => AirFnArgs::Add32Args(),
                WriteJsonSubCommand::BitUnpack => AirFnArgs::BitUnpackArgs(),
                WriteJsonSubCommand::Ret => AirFnArgs::RetOpcodeArgs(),
                WriteJsonSubCommand::AssertEqual(assert_eq_opcode_args) => {
                    AirFnArgs::AssertEqOpcodeArgs(assert_eq_opcode_args)
                }
                WriteJsonSubCommand::Call(call_opcode_args) => {
                    AirFnArgs::CallOpcodeArgs(call_opcode_args)
                }
                WriteJsonSubCommand::Jump(jump_opcode_args) => {
                    AirFnArgs::JumpOpcodeArgs(jump_opcode_args)
                }
            };
            println!("Input args {:?}", air_fn_args);
            dump_to_file(&create_air_fn_json(air_fn_args), None);
        }
    }
}
