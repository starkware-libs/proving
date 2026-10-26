use super::*;

/// The `ComponentList` index of every component.
fn indices() -> PerComponent<usize> {
    PerComponent {
        eq: ComponentList::Eq.idx(),
        qm_31_ops: ComponentList::Qm31Ops.idx(),
        triple_xor: ComponentList::TripleXor.idx(),
        m_31_to_u_32: ComponentList::M31ToU32.idx(),
        blake_g_gate: ComponentList::BlakeGGate.idx(),
        verify_bitwise_xor_8: ComponentList::VerifyBitwiseXor8.idx(),
        verify_bitwise_xor_12: ComponentList::VerifyBitwiseXor12.idx(),
        verify_bitwise_xor_4: ComponentList::VerifyBitwiseXor4.idx(),
        verify_bitwise_xor_7: ComponentList::VerifyBitwiseXor7.idx(),
        verify_bitwise_xor_9: ComponentList::VerifyBitwiseXor9.idx(),
        range_check_16: ComponentList::RangeCheck16.idx(),
    }
}

#[test]
fn test_per_component_helpers_keep_component_list_order() {
    let expected: [usize; N_COMPONENTS] = std::array::from_fn(|i| i);
    assert_eq!(indices().into_array(), expected);

    // `map` visits the components in `ComponentList` order.
    let mut visited = vec![];
    let doubled = indices().map(|idx| {
        visited.push(idx);
        2 * idx
    });
    assert_eq!(visited, expected);
    assert_eq!(doubled.into_array(), expected.map(|idx| 2 * idx));

    assert_eq!(indices().as_ref().map(|idx| *idx), indices());

    let zipped = indices().zip(indices().map(|idx| COMPONENT_NAMES[idx])).into_array();
    assert_eq!(zipped, std::array::from_fn(|i| (i, COMPONENT_NAMES[i])));

    let map = indices().into_index_map();
    assert!(map.keys().copied().eq(COMPONENT_NAMES));
    assert!(map.values().copied().eq(expected));
}
