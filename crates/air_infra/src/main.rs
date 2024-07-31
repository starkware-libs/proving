use clap::{Parser, Subcommand};

use air_infra::airs::examples::bit_unpacking::create_bit_unpacking_json;
use air_infra::airs::examples::fibonacci::create_fibonacci_json;
use air_infra::airs::uint32_utils::create_add32_json;
use air_infra::utils::fn_sizes::*;

#[derive(Subcommand, Debug)]
enum MainCommands {
    FnSizes,
    WriteJson { fn_name: String },
}

#[derive(Parser, Debug)]
struct MainArgs {
    #[command(subcommand)]
    command: MainCommands,
}

pub fn write_json(fn_name: String) {
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
        MainCommands::WriteJson { fn_name } => write_json(fn_name),
    };
}
