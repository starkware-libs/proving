pub mod range_check;
#[cfg(test)]
pub mod range_check_test;
pub mod seq;
pub mod verify_bitwise_xor;
#[cfg(test)]
pub mod verify_bitwise_xor_test;

pub(super) fn get_relation_variant_names(base_name: &str, n_variants: usize) -> Vec<String> {
    let mut names = Vec::with_capacity(n_variants);
    for i in 0..n_variants {
        if i == 0 {
            names.push(base_name.to_string());
        } else {
            names.push(format!("{}_{}", base_name, (b'B' + (i - 1) as u8) as char));
        }
    }
    names
}
