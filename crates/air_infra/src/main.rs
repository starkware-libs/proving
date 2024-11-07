use air_infra::utils::create_opcodes_json::*;
use air_infra::utils::fn_sizes::*;
use clap::{Parser, Subcommand};
use compiled_casm_air::utils::*;

#[derive(Subcommand, Debug)]
enum MainCommands {
    FnSizes,                     // Usage: cargo run fn-sizes
    WriteJson(WriteJsonCommand), // Usage: cargo run write-json opcode/example_name --<arguments>
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
            println!("Input args {:?}", command.args);
            dump_to_file(&create_air_fn_registry(command.args), None);
        }
    }
}
