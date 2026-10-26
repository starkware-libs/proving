use core::box::BoxImpl;
use crate::fields::m31::m31;
use crate::vcs::blake2s_hasher::{Blake2sHash, Blake2sMerkleHasher};

#[test]
fn test_hash_leaf_with_two_values() {
    assert_eq!(
        Blake2sMerkleHasher::hash_leaf(array![m31(0), m31(1)].span()).hash.unbox(),
        [
            3950351958, 4278888560, 2450494307, 4106812851, 2998960590, 1139581150, 933467563,
            4130483740,
        ],
    );
}

#[test]
fn test_hash_children() {
    let l_node = Blake2sHash {
        hash: BoxImpl::new(
            [
                3950351958, 4278888560, 2450494307, 4106812851, 2998960590, 1139581150, 933467563,
                4130483740,
            ],
        ),
    };
    let r_node = Blake2sHash {
        hash: BoxImpl::new(
            [
                707328262, 1356420215, 727373128, 3551204147, 4019359644, 4016890851, 1375080809,
                3547510316,
            ],
        ),
    };
    assert_eq!(
        Blake2sMerkleHasher::hash_children(l_node, r_node).hash.unbox(),
        [
            2021167039, 2235961952, 1255039539, 1377649483, 2239127732, 2718601205, 4011324288,
            2878352859,
        ],
    );
}

#[test]
fn test_hash_leaf_with_single_value() {
    assert_eq!(
        Blake2sMerkleHasher::hash_leaf(array![m31(5)].span()).hash.unbox(),
        [
            1762149166, 51569020, 2892006170, 256092287, 1833904684, 2685651037, 1017105109,
            832784341,
        ],
    );
}

#[test]
fn test_hash_leaf_with_qm31_value() {
    assert_eq!(
        Blake2sMerkleHasher::hash_leaf(array![m31(7), m31(8), m31(9), m31(10)].span()).hash.unbox(),
        [
            3046786000, 388266688, 1986680006, 3567470814, 1002034150, 2715825190, 3845047605,
            357639097,
        ],
    );
}

#[test]
fn test_hash_leaf_with_many_values() {
    let values = array![
        m31(0), m31(1), m31(2), m31(3), m31(4), m31(5), m31(6), m31(7), m31(8), m31(9), m31(10),
        m31(11), m31(12), m31(13), m31(14), m31(15), m31(16), m31(17), m31(18), m31(19), m31(20),
        m31(21),
    ];
    assert_eq!(
        Blake2sMerkleHasher::hash_leaf(values.span()).hash.unbox(),
        [
            4207748579, 9736850, 2660520810, 2447911355, 2122932718, 186156732, 3985413784,
            3584495042,
        ],
    );
}
