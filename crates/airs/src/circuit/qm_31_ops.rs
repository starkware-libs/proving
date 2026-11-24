use air_common::{TraceType, UseOrYield};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use serde::Serialize;

use super::ext_tables::*;
use crate::convolution_utils::qm31_mul::qm31_mul_expr_in_air;

#[derive(Clone, Debug, Serialize)]
pub struct Qm31Ops {}

impl AirFn for Qm31Ops {
    type ExtIn = ();
    type In = [[FeltExpr; 4]; 3];
    type Out = ();

    fn call(&self, ab: &mut AirBuilder, _: (), [op0, op1, dst]: Self::In) -> Self::Out {
        let add_flag = ab.call_external_table(&AddFlag {});
        let sub_flag = ab.call_external_table(&SubFlag {});
        let mul_flag = ab.call_external_table(&MulFlag {});
        let pointwise_mul_flag = ab.call_external_table(&PointwiseMulFlag {});

        // Unimodular Karatsuba-9 QM31 product over M31 (9 multiplications, 0 non-unit constants).
        let mul_result = qm31_mul_expr_in_air(ab, &op0, &op1);

        for i in 0..4 {
            ab.constrain(
                dst[i].clone()
                    - (mul_result[i].clone() * mul_flag.as_felt()
                        + (op0[i].clone() + op1[i].clone()) * add_flag.as_felt()
                        + (op0[i].clone() - op1[i].clone()) * sub_flag.as_felt()
                        + (op0[i].clone() * op1[i].clone()) * pointwise_mul_flag.as_felt()),
                "",
            )
        }

        let op0_addr = ab.call_external_table(&Op0Addr {});
        let op1_addr = ab.call_external_table(&Op1Addr {});
        let dst_addr = ab.call_external_table(&DstAddr {});
        let mult = ab.call_external_table(&QM31OpsMultiplicity {});

        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![op0_addr.var].into_iter().chain(op0).collect(),
            UseOrYield::Use,
            const_expr!(1),
        );
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![op1_addr.var].into_iter().chain(op1).collect(),
            UseOrYield::Use,
            const_expr!(1),
        );
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![dst_addr.var].into_iter().chain(dst.clone()).collect(),
            UseOrYield::Yield,
            mult,
        );
    }

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("op0_limb0".to_string()),
            Some("op0_limb1".to_string()),
            Some("op0_limb2".to_string()),
            Some("op0_limb3".to_string()),
            Some("op1_limb0".to_string()),
            Some("op1_limb1".to_string()),
            Some("op1_limb2".to_string()),
            Some("op1_limb3".to_string()),
            Some("dst_limb0".to_string()),
            Some("dst_limb1".to_string()),
            Some("dst_limb2".to_string()),
            Some("dst_limb3".to_string()),
        ])
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Gate
    }
}
