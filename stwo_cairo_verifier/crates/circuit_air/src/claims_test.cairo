use core::dict::{Felt252DictTrait, SquashedFelt252DictTrait};
use stwo_constraint_framework::test_utils::{RelationUsesDict, accumulate_relation_uses};
use stwo_verifier_core::fields::m31::P_U32;
use stwo_verifier_utils::zip_eq::zip_eq;
use crate::components;
use crate::multiverifier_consts::COMPONENT_LOG_SIZES;
use crate::per_component::{PerComponent, PerComponentTrait};

/// Checks that, for every lookup relation, the total number of uses across all components is
/// less than `P`.
///
/// The circuit is fixed-size, so the uses are determined by the hardcoded
/// `COMPONENT_LOG_SIZES` (that's why this is a test rather than a verifier-side check).
#[test]
fn relation_uses_are_below_p() {
    let relation_uses_per_row = PerComponent {
        eq: components::eq::RELATION_USES_PER_ROW.span(),
        qm_31_ops: components::qm_31_ops::RELATION_USES_PER_ROW.span(),
        triple_xor: components::triple_xor::RELATION_USES_PER_ROW.span(),
        m_31_to_u_32: components::m_31_to_u_32::RELATION_USES_PER_ROW.span(),
        blake_g_gate: components::blake_g_gate::RELATION_USES_PER_ROW.span(),
        verify_bitwise_xor_8: components::verify_bitwise_xor_8::RELATION_USES_PER_ROW.span(),
        verify_bitwise_xor_12: components::verify_bitwise_xor_12::RELATION_USES_PER_ROW.span(),
        verify_bitwise_xor_4: components::verify_bitwise_xor_4::RELATION_USES_PER_ROW.span(),
        verify_bitwise_xor_7: components::verify_bitwise_xor_7::RELATION_USES_PER_ROW.span(),
        verify_bitwise_xor_9: components::verify_bitwise_xor_9::RELATION_USES_PER_ROW.span(),
        range_check_16: components::range_check_16::RELATION_USES_PER_ROW.span(),
    };

    let mut relation_uses: RelationUsesDict = Default::default();
    for (uses_per_row, log_size) in zip_eq(
        relation_uses_per_row.to_fixed_array().span(), COMPONENT_LOG_SIZES.to_fixed_array().span(),
    ) {
        accumulate_relation_uses(ref relation_uses, *uses_per_row, *log_size);
    }

    let squashed = relation_uses.squash();
    for entry in squashed.into_entries() {
        let (_relation_id, _first_uses, last_uses) = entry;
        assert!(last_uses < P_U32.into(), "A relation has more than P-1 uses");
    }
}
