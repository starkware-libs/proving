use num_traits::One;
use stwo::core::fields::cm31::CM31;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::ivalue::{IValue, qm31_from_u32s};

#[test]
fn test_pointwise_inv_or_zero() {
    let a = qm31_from_u32s(2, 0, 3, 4);
    let a_inv = a.pointwise_inv_or_zero();
    let one = M31::one();
    assert_eq!(
        a_inv,
        QM31(CM31(one / M31::from(2), 0.into()), CM31(one / M31::from(3), one / M31::from(4)))
    );
}
