/// Returns the base-2 logarithm of `x`, or `None` if `x` is not a power of two.
pub fn safe_log2(x: u128) -> Option<u32> {
    x.is_power_of_two().then(|| x.trailing_zeros())
}
