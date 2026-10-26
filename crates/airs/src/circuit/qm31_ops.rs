use air_common::{TraceType, UseOrYield};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use serde::Serialize;

use super::ext_tables::*;

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

        ab.constrain(
            add_flag.as_felt()
                + sub_flag.as_felt()
                + mul_flag.as_felt()
                + pointwise_mul_flag.as_felt()
                - const_expr!(1),
            "all flags sum to 1",
        );
        ab.constrain(
            add_flag.as_felt() * (add_flag.as_felt() - const_expr!(1)),
            "add_flag is a bit",
        );
        ab.constrain(
            sub_flag.as_felt() * (sub_flag.as_felt() - const_expr!(1)),
            "sub_flag is a bit",
        );
        ab.constrain(
            mul_flag.as_felt() * (mul_flag.as_felt() - const_expr!(1)),
            "mul_flag is a bit",
        );
        ab.constrain(
            pointwise_mul_flag.as_felt() * (pointwise_mul_flag.as_felt() - const_expr!(1)),
            "pointwise_mul_flag is a bit",
        );

        // When expanding (a0+b0i+c0j+d0k) * (a1+b1i+c1j+d1k)
        // and regrouping as coordinates in (1, i, j, k) we arrive at the result
        // a0 * a1 - b0 * b1 + 2*(c0*c1 - d0*d1) - c0*d1 - d0*c1
        // + i*(a0 * b1 + b0 * a1 + 2*(c0*d1 + d0*c1) + c0*c1 - d0*d1)
        // + j*(a0 * c1 - b0 * d1 + c0 * a1 - d0 * b1)
        // + k*(a0 * d1 + b0 * c1 + c0 * b1 + d0 * a1)
        // Hence, the coordinates in mul_result are those of op0 * op1
        let mul_result = [
            op0[0].clone() * op1[0].clone() - op0[1].clone() * op1[1].clone()
                + const_expr!(2)
                    * (op0[2].clone() * op1[2].clone() - op0[3].clone() * op1[3].clone())
                - op0[2].clone() * op1[3].clone()
                - op0[3].clone() * op1[2].clone(),
            op0[0].clone() * op1[1].clone()
                + op0[1].clone() * op1[0].clone()
                + const_expr!(2)
                    * (op0[2].clone() * op1[3].clone() + op0[3].clone() * op1[2].clone())
                + op0[2].clone() * op1[2].clone()
                - op0[3].clone() * op1[3].clone(),
            op0[0].clone() * op1[2].clone() - op0[1].clone() * op1[3].clone()
                + op0[2].clone() * op1[0].clone()
                - op0[3].clone() * op1[1].clone(),
            op0[0].clone() * op1[3].clone()
                + op0[1].clone() * op1[2].clone()
                + op0[2].clone() * op1[1].clone()
                + op0[3].clone() * op1[0].clone(),
        ];

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
