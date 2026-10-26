use crate::deconstruct_f252;

#[test]
fn test_deconstruct_felt() {
    assert_eq!(
        deconstruct_f252(0x800000007000000060000000500000004000000030000000200000001).unbox(),
        [1_u32, 2, 3, 4, 5, 6, 7, 8],
    );
}
