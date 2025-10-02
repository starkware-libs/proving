pub mod create_opcodes_json;
pub mod fn_sizes;
#[cfg(test)]
pub mod test_utils;

use convert_case::{Case, Casing};

pub fn is_false(b: &bool) -> bool {
    !b
}

pub fn fix_str(mut name: String) -> String {
    while name.contains("__") {
        name = name.replace("__", "_");
    }
    if name.ends_with('_') {
        name.pop();
    }
    if name.starts_with('_') {
        name = name[1..].to_string();
    }
    name.to_case(Case::Snake)
}
