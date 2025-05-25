use std::fmt::Display;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::{fs, io};

use convert_case::{Case, Casing};
use serde::Serialize;
use serde_json::{to_writer_pretty, Value};

use super::compiled_structs::*;

pub const JSONS_OPCODES_DIR: &str = "../compiled_casm_air/src/opcodes/";
pub const JSONS_BUILTINS_DIR: &str = "../compiled_casm_air/src/builtins/";
pub const JSONS_LOOKUPS_DIR: &str = "../compiled_casm_air/src/lookups/";
pub const JSONS_INLINE_DIR: &str = "../compiled_casm_air/src/subroutines/";

pub const INTERMEDIATE_VAR_SUFFIX: &str = "tmp";
pub const STATE_VAR_SUFFIX: &str = "col";
pub const INPUT_VAR_SUFFIX: &str = "input";
pub const OUTPUT_VAR_SUFFIX: &str = "output";
pub const STATE_INPUT_VAR: &str = "input";
pub const STATE_OUTPUT_VAR_SUFFIX: &str = "output";

pub const WRITE_TRACE_FUNCTION_NAME: &str = "write_trace";
pub const CONSTRAINT_EVAL_FUNCTION_NAME: &str = "evaluate";

impl Display for CompiledAirVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompiledAirVar::StaticCall(id, args) => {
                write!(f, "{}(", id)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            CompiledAirVar::MethodCall(left, id, args) => {
                write!(f, "{}.{}(", left, id)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            CompiledAirVar::Const(_, id) => write!(f, "{}", id),
            CompiledAirVar::Var(_, id) => {
                write!(f, "{}", id)
            }
            CompiledAirVar::State(name) => {
                write!(f, "{}", name)
            }
            CompiledAirVar::BinaryOp(lhs, op, rhs) => {
                write!(f, "({} {} {})", lhs, op, rhs)
            }
            CompiledAirVar::UnaryOp(op, expr) => {
                write!(f, "({} {})", op, expr)
            }
            CompiledAirVar::Tuple(exprs) => {
                let strs = exprs.iter().map(ToString::to_string).collect::<Vec<_>>();
                write!(f, "{}", &format!("({})", strs.join(", ")))
            }
            CompiledAirVar::Array(exprs) => {
                write!(f, "{}", vars_arr_to_string(exprs))
            }
            CompiledAirVar::Struct { fields, .. } => {
                let strs = fields
                    .iter()
                    .map(|(name, expr)| format!("{}: {}", name, expr))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{{}}}", strs)
            }
            CompiledAirVar::ExternalState(ExternalState {
                name,
                generic_param: _,
                args,
            }) => {
                write!(f, "{}({})", name.to_case(Case::Snake), args.join(", "))
            }
            CompiledAirVar::PublicParam(name) => {
                write!(f, "public_params.{}", name)
            }
        }
    }
}

pub fn vars_arr_to_string(felts: &[CompiledAirVar]) -> String {
    let mut strs = felts.iter().map(ToString::to_string).collect::<Vec<_>>();
    let mut i = 0;
    let mut leading_zeros = false;
    for s in strs.iter().rev() {
        if s == "0" {
            leading_zeros = true;
            i += 1;
        } else {
            break;
        }
    }
    strs.truncate(strs.len() - i);
    let str = format!("[{}]", strs.join(", "));
    if leading_zeros {
        format!("zero_extend({})", str)
    } else {
        str
    }
}

pub fn project_root() -> PathBuf {
    std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
}

// Dumps the sereilazied object to a file. If there is no path given, it will dump to stdout.
pub fn dump_to_file<T>(value: &T, path: Option<&str>)
where
    T: Serialize,
{
    let mut writer: Box<dyn Write> = match path {
        Some(p) => {
            let file = File::create(project_root().join(p)).expect("Unable to create file");
            Box::new(BufWriter::new(file)) // Write to file
        }
        None => {
            Box::new(BufWriter::new(io::stdout())) // Write to stdout
        }
    };
    to_writer_pretty(&mut writer, value).expect("serialization failed");
    writer.flush().expect("flush failed");
    writer.write_all(b"\n").expect("write failed");
}

pub fn read_json(file_path: &str) -> Value {
    let json_file =
        fs::read_to_string(file_path).unwrap_or_else(|e| panic!("Failed to read {file_path}: {e}"));
    serde_json::from_str(&json_file).expect("Invalid JSON file")
}
