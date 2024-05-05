use std::any::type_name;
use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use super::air_fn_registry::*;
#[cfg(test)]
use super::expressions::expr::*;
use super::expressions::felt_expr::*;
use super::state::*;
use super::variables::*;

pub const CONSTRAINT_INTERMEDIATE_VAR_PREFIX: &str = "constraint_tmp_";
pub const DEDUCTION_INTERMEDIATE_VAR_PREFIX: &str = "deduction_tmp_";

// An air function should define a struct that implements the AirFn trait.
// The AirFn trait has two associated types, In and Out, which are the input and output types of the
// air function. It also defines whether the input is in the trace or not.
// The call method is the main method of the air function, and is used to build and run the air
// function.
pub trait AirFn: Debug {
    type In: AirVar;
    type Out: AirVar;

    fn name(&self) -> String {
        let mut name = type_name::<Self>().to_string();
        name = name
            .rfind("::")
            .map(|i| name[i + 2..].to_string())
            .unwrap_or(name);
        self.inst_def().iter().for_each(|(_, v)| {
            name.push_str(format!("__{}", v).as_str());
        });
        name.to_string()
    }

    fn input_in_trace(&self) -> bool;
    fn inst_def(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out;
}

// AirBuilder is a struct that is used to build an air function.
// It is passed to the call method of an air function, and is used to add constraints, deductions,
// assignments and intermediate variables to the air function.
#[derive(Debug)]
pub struct AirBuilder {
    pub(super) state: State,
    pub(super) air_body: Vec<AirBodyComponent>,
    #[cfg(test)]
    pub(super) run: bool,
    pub(super) registry: AirFnRegistry,
}
impl AirBuilder {
    pub fn constrain(&mut self, expr: FeltExpr) {
        assert!(
            expr.in_state(),
            "The mask of the constraint must be in the trace."
        );

        #[cfg(test)]
        if self.run {
            assert!(
                expr.calc() == 0.to_string(),
                "Added incorrect constraint (does not evalutate to 0)"
            )
        }

        self.air_body.push(AirBodyComponent::Constraint(expr));
    }
    pub fn deduce(&mut self, expr: &mut FeltExpr) -> FeltExpr {
        self.air_body
            .push(AirBodyComponent::Deduction(expr.clone()));
        self.state.add(expr);
        expr.clone()
    }
    pub fn assign(&mut self, expr: &mut FeltExpr) -> FeltExpr {
        let before = expr.clone();
        self.state.add(expr);

        let constraint = &*expr - &before;
        self.air_body.push(AirBodyComponent::Assignment {
            constraint: constraint.clone(),
            deduction: before,
        });
        expr.clone()
    }

    pub fn create_intermediate_var_for_deduction<V>(&mut self, var: V) -> V
    where
        V: AirVar,
    {
        let index = self.registry.get_intermediate_var_index();
        let name = format!("{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, index);
        self.air_body.push(AirBodyComponent::DeductionIntermediate(
            name.clone(),
            var.clone().into(),
        ));
        var.create_intermediate_var_for_deduction(name)
    }

    pub fn let_for_constraint(&mut self, expr: &FeltExpr) -> FeltExpr {
        let index = self.registry.get_intermediate_var_index();
        let name = format!("{}{}", CONSTRAINT_INTERMEDIATE_VAR_PREFIX, index);
        self.air_body.push(AirBodyComponent::ConstraintIntermediate(
            name.clone(),
            expr.clone(),
        ));
        expr.let_for_constraint(name)
    }

    pub fn call<I, O>(&mut self, air_fn: &dyn AirFn<In = I, Out = O>, input: I) -> O
    where
        I: AirVar,
        O: AirVar,
    {
        if air_fn.input_in_trace() {
            assert!(input.in_state(), "Input must be in the trace");
        }
        if self.registry.air_fns.borrow().get(&air_fn.name()).is_none() {
            AirFnEntry::new(&self.registry, air_fn);
        }

        let mut air_builder = Self {
            state: self.state.clone(),
            air_body: vec![],
            #[cfg(test)]
            run: self.run,
            registry: self.registry.clone(),
        };
        let output = air_fn.call(&mut air_builder, input.clone());
        self.air_body.push(AirBodyComponent::Subroutine(Call {
            air_fn_name: air_fn.name(),
            input_arg: input.into(),
            output: output.clone().into(),
            state: air_builder.state,
            air_body: air_builder.air_body,
        }));
        output
    }
}

// A Call is an air_body component that represents a call to another air function.
// It contains the name of the air function, the input argument, the output of the call, the state
// after the call, and the air_body of the called function.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Call {
    pub air_fn_name: String,
    pub input_arg: GenericAirVar,
    pub output: GenericAirVar,
    #[serde(skip)]
    pub state: State,
    #[serde(skip)]
    pub air_body: Vec<AirBodyComponent>,
}

// Each air function has an air_body, which is a vector of AirBodyComponent.
// These are the components of the air function.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AirBodyComponent {
    Constraint(FeltExpr),
    Deduction(FeltExpr),
    // An assignment is a constraint and a deduction referring to the same trace cell.
    // For example, when copying a value from one trace cell to another.
    Assignment {
        constraint: FeltExpr,
        deduction: FeltExpr,
    },
    DeductionIntermediate(String, GenericAirVar),
    ConstraintIntermediate(String, FeltExpr),
    Subroutine(Call),
}
