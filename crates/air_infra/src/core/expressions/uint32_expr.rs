use stwo_cairo_common::prover_types::cpu::UInt32;

use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::uint16_expr::*;
use super::var_expr::*;

pub type UInt32Operation = OpExpr<UInt32>;
pub type UInt32Expr = Expr<UInt32>;
const LOW_NAME: &str = "low";
const HIGH_NAME: &str = "high";

impl VarExpr<UInt32> {
    fn get_children(&mut self) -> [&mut UInt16Expr; 2] {
        self.complex_or_felt
            .as_complex_mut()
            .iter_mut()
            .map(|c| match c {
                ExprImpl::UInt16(e) => e,
                _ => panic!("Invalid child type"),
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("UInt32 var must have 2 uint16 children.")
    }

    fn get_child_mut(&mut self, index: usize) -> &mut UInt16Expr {
        match self
            .complex_or_felt
            .as_complex_mut()
            .get_mut(index)
            .expect("Invalid index")
        {
            ExprImpl::UInt16(e) => e,
            _ => panic!("Invalid child type"),
        }
    }

    fn get_child(&self, index: usize) -> UInt16Expr {
        match self
            .complex_or_felt
            .as_complex()
            .get(index)
            .expect("Invalid index")
        {
            ExprImpl::UInt16(e) => e.clone(),
            _ => panic!("Invalid child type"),
        }
    }
}

impl VarExprUpdate for VarExpr<UInt32> {
    fn create_children(&mut self) {
        let low = VarExpr::new(
            LOW_NAME.to_string(),
            self.value.map(|v| v.low()),
            self.is_const,
            self.in_state(),
        );
        let high = VarExpr::new(
            HIGH_NAME.to_string(),
            self.value.map(|v| v.high()),
            self.is_const,
            self.in_state(),
        );
        self.complex_or_felt = ComplexOrFelt::Complex(vec![
            UInt16Expr::Var(low).into(),
            UInt16Expr::Var(high).into(),
        ]);
    }

    fn update_children(&mut self) {
        let parent_var = &self.clone();
        self.get_child_mut(0)
            .as_var_mut()
            .set_parent(parent_var, None);
        self.get_child_mut(1)
            .as_var_mut()
            .set_parent(parent_var, None);
    }
}

impl UInt32Expr {
    pub fn low_mut(&mut self) -> &mut UInt16Expr {
        self.as_var_mut().get_child_mut(0)
    }

    pub fn high_mut(&mut self) -> &mut UInt16Expr {
        self.as_var_mut().get_child_mut(1)
    }

    pub fn low(&self) -> UInt16Expr {
        self.as_var().get_child(0)
    }

    pub fn high(&self) -> UInt16Expr {
        self.as_var().get_child(1)
    }
}

impl AirVar for UInt32Expr {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            UInt32Expr::Var(v) => v
                .get_children()
                .into_iter()
                .flat_map(|e| e.as_felts_mut())
                .collect(),
            UInt32Expr::Op(op) => {
                if op.op == Operation::UInt32FromFeltsPair {
                    if let [AirVarImpl::Expr(ExprImpl::Felt(felt1)), AirVarImpl::Expr(ExprImpl::Felt(felt2))] =
                        &mut op.children[..]
                    {
                        return vec![felt1, felt2];
                    }
                }
                panic!("Cannot convert to felts");
            }
        }
    }

    fn get_felt_descriptions(&self) -> Option<Vec<String>> {
        Some(vec!["low".to_string(), "high".to_string()])
    }
}

#[macro_export]
macro_rules! const_u32_expr {
    ($val:expr) => {
        UInt32Expr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            $val.into(),
        ))
    };
}

#[cfg(test)]
macro_rules! u32_expr {
    ($name:expr, $val:expr) => {
        UInt32Expr::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some(UInt32::from($val)),
            false,
            false,
        ))
    };
}
#[cfg(test)]
pub(super) use u32_expr;
