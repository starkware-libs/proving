use std::fs;

use compiled_casm_air::utils::dump_to_file;
use serde::Serialize;

pub const TEST_JSONS_EXAMPLES_DIR: &str = "src/airs/examples/test_jsons/";

pub fn compare_json<T>(value: &T, file_path: &String)
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
            file_path
        );
    };
}
