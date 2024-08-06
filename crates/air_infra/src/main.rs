use clap::{Parser, Subcommand};

use air_infra::airs::examples::bit_unpacking::create_bit_unpacking_json;
use air_infra::airs::examples::fibonacci::create_fibonacci_json;
use air_infra::airs::uint32_utils::create_add32_json;
use air_infra::utils::create_opcodes_json::*;
use air_infra::utils::fn_sizes::*;

#[derive(Subcommand, Debug)]
enum MainCommands {
    FnSizes,
    WriteJsonExample {
        fn_name: String,
    }, // Usage: cargo run writre-json-example <function_name>
    WriteJsonCasmOpcode {
        fn_name: String,
        opcode_args: Vec<bool>,
    }, // Usage: cargo run write-json-casm-opcode <function_name> <arguments>
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

pub fn write_json_casm_opcdode(fn_name: String, opcode_args: Vec<bool>) {
    if fn_name == "assert_eq" {
        assert!(
            opcode_args.len() == 2,
            "Expected 2 arguments for assert_eql: is_double_deref, is_immediate"
        );
        println!("Creating a json file for assert equal opcode");
        create_assert_equal_opcdode_json(opcode_args);
    } else if fn_name == "call" {
        assert!(
            opcode_args.len() == 2,
            "Expected 2 arguments for call: is_rel, flag_op1_base_fp"
        );
        println!("Creating a json file for call opcode");
        create_call_opcdode_json(opcode_args);
    } else if fn_name == "jump" {
        assert!(
            opcode_args.len() == 3,
            "Expected 3 arguments for jump: is_rel, flag_op1_base_fp, flag_ap_update_add_1"
        );
        println!("Creating a json file for jump opcode");
        create_jump_opcdode_json(opcode_args);
    } else if fn_name == "ret" {
        assert!(opcode_args.is_empty(), "Expected 0 arguments for ret");
        println!("Creating a json file for ret opcode");
        create_ret_opcdode_json();
    }
}

pub fn main() {
    let args = MainArgs::parse();

    match args.command {
        MainCommands::FnSizes => print_fn_sizes(),
        MainCommands::WriteJsonExample { fn_name } => write_json_example(fn_name),
        MainCommands::WriteJsonCasmOpcode {
            fn_name,
            opcode_args,
        } => write_json_casm_opcdode(fn_name, opcode_args),
    };
}
