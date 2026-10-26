use crate::fields::m31::m31;
use crate::vcs::poseidon_hasher::PoseidonMerkleHasher;

#[test]
fn test_m31() {
    assert_eq!(
        PoseidonMerkleHasher::hash_leaf(array![m31(0), m31(1)].span()),
        2978883932528585652864046122079599882777358126302490183268546077323303473078,
    );
}

/// The expected values were computed with stwo's `Poseidon252MerkleHasher::hash_children`.
#[test]
fn test_hash_children() {
    assert_eq!(
        PoseidonMerkleHasher::hash_children(1, 2),
        2636648219362971850283425434366427370362725365790740855428580782178634926362,
    );
    assert_eq!(
        PoseidonMerkleHasher::hash_children(2, 1),
        1495775059983760120370762865285436974484912473388770620705938782644775396833,
    );
}
