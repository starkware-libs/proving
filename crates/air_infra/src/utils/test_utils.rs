use serde_json::Value;
use std::fs;

use crate::core::air_fn_registry::*;

#[cfg(test)]
pub const TEST_JSONS_OPCODES_DIR: &str = "src/airs/casm/opcodes/test_jsons/";
#[cfg(test)]
pub const TEST_JSONS_BUILTINS_DIR: &str = "src/airs/casm/builtins/test_jsons/";
#[cfg(test)]
pub const TEST_JSONS_CONST_TABLES_DIR: &str = "src/airs/casm/const_tables/test_jsons/";
#[cfg(test)]
pub const TEST_JSONS_EXAMPLES_DIR: &str = "src/airs/examples/test_jsons/";
#[cfg(test)]
pub const TEST_JSONS_FELT252_DIR: &str = "src/airs/felt252_utils/test_jsons/";
#[cfg(test)]
pub const TEST_JSONS_MEMORY_DIR: &str = "src/airs/memory/test_jsons/";
#[cfg(test)]
pub const TEST_JSONS_UINT32_DIR: &str = "src/airs/uint32_utils/test_jsons/";

pub fn compare_test_json(registry: AirFnRegistry, air_fn_name: &String, file_path: &String) {
    let is_fix_mode = std::env::var("FIX") == Ok("1".to_string());
    if is_fix_mode {
        registry.dump_to_file(Some(air_fn_name), Some(file_path));
    } else {
        let json_file = fs::read_to_string(file_path.clone()).unwrap();
        let expected_entry_json: Value =
            serde_json::from_str(&json_file).expect("Invalid JSON file for the expected entry");
        let entry = registry.get_air_fn_entry(air_fn_name);
        let entry_json =
            serde_json::to_value(&entry).expect("Failed to convert current entry to JSON value");
        assert!(
            entry_json == expected_entry_json,
            r#"
            Generated entry json for {}
            is different from the entry in {}.
            Run the following to update the code:
            '$ FIX=1 cargo test'"#,
            air_fn_name,
            file_path
        );
    };
}
