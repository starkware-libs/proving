use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{fs, io};

use air_common::utils::project_root;
use air_common::{SAMPLE_EVALUATIONS_FILE_NAME, TraceType};
use eval_air_fn_constraints::create_sample_evaluation;
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::to_writer_pretty;

use crate::core::air_fn_registry::AirFnRegistry;

pub const JSONS_OPCODES_DIR: &str = "compiled_jsons/opcodes/";
pub const JSONS_BUILTINS_DIR: &str = "compiled_jsons/builtins/";
pub const JSONS_LOOKUPS_DIR: &str = "compiled_jsons/lookups/";
pub const JSONS_INLINE_DIR: &str = "compiled_jsons/subroutines/";
pub const JSONS_GATES_DIR: &str = "compiled_jsons/gates/";

pub fn compare_json<T>(value: &T, file_path: &Path)
where
    T: Serialize,
{
    let is_fix_mode = std::env::var("FIX") == Ok("1".to_string());
    if is_fix_mode {
        dump_to_file(value, Some(file_path));
    } else {
        // It is more efficient to compare strings than jsons.
        let expected =
            fs::read_to_string(file_path).expect("Failed to read the expected JSON file");
        let mut given = serde_json::to_string_pretty(value).expect("serialization failed");
        given.push('\n');

        assert!(
            given == expected,
            r#"
            Given value
            is different from the json in {}.
            Run the following to update the code:
            '$ FIX=1 cargo test'"#,
            file_path.display()
        );
    };
}

// Dumps the sereilazied object to a file. If there is no path given, it will dump to stdout.
pub fn dump_to_file<T>(value: &T, path: Option<&Path>)
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

// Compiles the registry, samples evaluations, and checks the jsons.
pub fn compare_registry_jsons(registry: &AirFnRegistry, path: &Path) {
    let compiled_reg = registry.compile();
    let mut sample_evaluations = IndexMap::new();

    for (name, compiled_entry) in compiled_reg.iter() {
        // Const functions are not compiled.
        if compiled_entry.r#type == TraceType::Const || compiled_entry.r#type == TraceType::Relation
        {
            continue;
        }

        let dir = match compiled_entry.r#type {
            TraceType::Opcode => JSONS_OPCODES_DIR,
            TraceType::Component | TraceType::Memory | TraceType::ChainRound => JSONS_LOOKUPS_DIR,
            TraceType::Builtin => JSONS_BUILTINS_DIR,
            TraceType::Inline => JSONS_INLINE_DIR,
            TraceType::Gate => JSONS_GATES_DIR,
            TraceType::Const | TraceType::Relation => {
                panic!("Const functions and relations should have been filtered out above")
            }
        };

        // Check the compiled entry json.
        compare_json(compiled_entry, &path.join(dir).join(format!("{name}.json")));

        if compiled_entry.r#type != TraceType::Inline {
            // We don't support sampling inline AirFns because they don't have a trace
            // of their own. This is OK because they are called by AirFns that we do support,
            // so their polynomial is tested as part of their caller.
            sample_evaluations
                .insert(name.to_string(), create_sample_evaluation(&compiled_reg, name));
        }
    }

    compare_json(&sample_evaluations, &path.join(SAMPLE_EVALUATIONS_FILE_NAME));
}
