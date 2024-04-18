#[cfg(test)]
mod tests {
    use air_infra::core::prover_types::Felt;
    use stwo::core::fields::m31::BaseField;

    #[test]
    fn test_felt_to_base_field() {
        let felt = Felt { value: 5 };

        let base_field: BaseField = felt.into();

        assert_eq!(base_field, BaseField::from_u32_unchecked(5));
    }
}
