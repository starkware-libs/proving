pub mod create_opcodes_json;
pub mod fn_sizes;
#[cfg(test)]
pub mod test_utils;

pub fn is_false(b: &bool) -> bool {
    !b
}
