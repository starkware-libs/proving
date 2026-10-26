macro_rules! define_component_list {
    ($($variant:ident => $field:ident),* $(,)?) => {
        pub enum ComponentList {
            $($variant),*
        }

        pub const N_COMPONENTS: usize = [$(stringify!($variant)),*].len();
        /// Canonical component names, in `ComponentList` order. These are the keys used for
        /// the per-component log-size map and match the keys of `all_circuit_components`.
        pub const COMPONENT_NAMES: [&str; N_COMPONENTS] = [$(stringify!($field)),*];

        impl ComponentList {
            /// The index of this component, in the static circuit components array.
            pub const fn idx(self) -> usize {
                self as usize
            }

            /// The canonical name of this component, used as the key in the per-component
            /// log-size map.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($field)),*
                }
            }
        }

        /// A value of type `T` for every circuit component, in canonic order, keyed by component name.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub struct PerComponent<T> {
            $(pub $field: T),*
        }

        impl<T> PerComponent<T> {
            /// Consumes `self`, returning the per-component values in `ComponentList` order.
            pub fn into_array(self) -> [T; N_COMPONENTS] {
                [$(self.$field),*]
            }

            /// Consumes `self`, yielding (name, value) pairs in `ComponentList` order.
            pub fn into_named_iter(self) -> impl Iterator<Item = (&'static str, T)> {
                itertools::zip_eq(COMPONENT_NAMES, self.into_array())
            }
        }
    };
}

define_component_list! {
    Eq => eq,
    Qm31Ops => qm31_ops,
    TripleXor => triple_xor,
    M31ToU32 => m_31_to_u_32,
    BlakeGGate => blake_g_gate,
    VerifyBitwiseXor8 => verify_bitwise_xor_8,
    VerifyBitwiseXor12 => verify_bitwise_xor_12,
    VerifyBitwiseXor4 => verify_bitwise_xor_4,
    VerifyBitwiseXor7 => verify_bitwise_xor_7,
    VerifyBitwiseXor9 => verify_bitwise_xor_9,
    RangeCheck16 => range_check_16,
}
