use serde::Serialize;

use compiled_casm_air::utils::{dump_to_file, read_json};

pub const TEST_JSONS_DECODE_INSTRUCTION_DIR: &str = "src/airs/casm/decode_instruction/test_jsons/";
pub const TEST_JSONS_EXAMPLES_DIR: &str = "src/airs/examples/test_jsons/";
pub const TEST_JSONS_FELT252_DIR: &str = "src/airs/felt252_utils/test_jsons/";
pub const TEST_JSONS_MEMORY_DIR: &str = "src/airs/felt252_id_memory/test_jsons/";
pub const TEST_JSONS_UINT32_DIR: &str = "src/airs/uint32_utils/test_jsons/";

pub fn compare_json<T>(value: &T, file_path: &String)
where
    T: Serialize,
{
    let is_fix_mode = std::env::var("FIX") == Ok("1".to_string());
    if is_fix_mode {
        dump_to_file(value, Some(file_path));
    } else {
        let expected_json = read_json(file_path);
        let given_json = serde_json::to_value(value).expect("Failed to serialize the given value");
        assert!(
            given_json == expected_json,
            r#"
            Given value
            is different from the json in {}.
            Run the following to update the code:
            '$ FIX=1 cargo test'"#,
            file_path
        );
    };
}
