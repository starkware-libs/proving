use serde::Serialize;
use serde_json::Value;
use std::fs;

use crate::core::utils::dump_to_file;

pub const TEST_JSONS_DECODE_INSTRUCTION_DIR: &str = "src/airs/casm/decode_instruction/test_jsons/";
pub const TEST_JSONS_OPCODES_DIR: &str = "src/airs/casm/opcodes/test_jsons/";
pub const TEST_JSONS_BUILTINS_DIR: &str = "src/airs/casm/builtins/test_jsons/";
pub const TEST_JSONS_EXAMPLES_DIR: &str = "src/airs/examples/test_jsons/";
pub const TEST_JSONS_FELT252_DIR: &str = "src/airs/felt252_utils/test_jsons/";
pub const TEST_JSONS_MEMORY_DIR: &str = "src/airs/memory/test_jsons/";
pub const TEST_JSONS_UINT32_DIR: &str = "src/airs/uint32_utils/test_jsons/";

pub fn compare_json<T>(value: &T, file_path: &String)
where
    T: Serialize,
{
    let is_fix_mode = std::env::var("FIX") == Ok("1".to_string());
    if is_fix_mode {
        dump_to_file(value, Some(file_path));
    } else {
        let json_file = fs::read_to_string(file_path.clone()).unwrap();
        let expected_entry_json: Value =
            serde_json::from_str(&json_file).expect("Invalid JSON file for the expected entry");
        let entry_json =
            serde_json::to_value(value).expect("Failed to convert current entry to JSON value");
        assert!(
            entry_json == expected_entry_json,
            r#"
            Given value
            is different from the json in {}.
            Run the following to update the code:
            '$ FIX=1 cargo test'"#,
            file_path
        );
    };
}
