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

// An AirFn that is intended to be a separate component in the trace and be called
// using lookup_call
pub trait LookupAirFn: AirFn<In = Self::InL, Out = Self::OutL> {
    // These are called InL and OutL instead of In, Out to not shadow the In, Out types in AirFn
    type InL: AirVar;
    type OutL: AirVar;

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        input: <Self as AirFn>::In,
    ) -> <Self as AirFn>::Out;

    fn inst_def(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }
}

impl<A> AirFn for A
where
    A: LookupAirFn,
{
    type In = <Self as LookupAirFn>::InL;
    type Out = <Self as LookupAirFn>::OutL;

    fn input_in_trace(&self) -> bool {
        false
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        <Self as LookupAirFn>::inst_def(self)
    }

    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out {
        let mut input_in_state = input.clone();
        input_in_state = air_builder.let_for_deduction(input_in_state);
        for felt in input_in_state.as_felts() {
            air_builder.deduce(felt);
        }
        <Self as LookupAirFn>::call(self, air_builder, input_in_state)
    }
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
        #[cfg(test)]
        if self.run {
            assert!(
                expr.in_state(),
                "The mask of the constraint must be in the trace."
            );

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
        #[cfg(test)]
        if self.run {
            assert!(
                expr.in_state(),
                "The mask of the constraint must be in the trace."
            );
        }

        let before = expr.clone();
        self.state.add(expr);

        let constraint = &*expr - &before;
        self.air_body.push(AirBodyComponent::Assignment {
            constraint: constraint.clone(),
            deduction: before,
        });
        expr.clone()
    }

    pub fn let_for_deduction<V>(&mut self, var: V) -> V
    where
        V: AirVar,
    {
        let index = self.registry.get_intermediate_var_index();
        let name = format!("{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, index);
        self.air_body.push(AirBodyComponent::DeductionIntermediate(
            name.clone(),
            var.clone().into(),
        ));
        var.let_for_deduction(name)
    }

    pub fn let_for_constraint(&mut self, expr: &FeltExpr) -> FeltExpr {
        #[cfg(test)]
        if self.run {
            assert!(
                expr.in_state(),
                "The mask of the intermediate variable for constraints must be in the trace."
            );
        }
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
        #[cfg(test)]
        if self.run && air_fn.input_in_trace() {
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
        self.air_body.push(AirBodyComponent::Call(Call {
            air_fn_name: air_fn.name(),
            input_arg: input.into(),
            output: output.clone().into(),
            air_body: air_builder.air_body,
        }));
        output
    }

    pub fn lookup_call<I, O>(
        &mut self,
        air_fn: &dyn LookupAirFn<In = I, Out = O, InL = I, OutL = O>,
        mut input: I,
    ) -> O
    where
        I: AirVar,
        O: AirVar,
    {
        // Make sure the callee is in the registry
        if self
            .registry
            .air_fns
            .borrow()
            .get(&(air_fn.name()))
            .is_none()
        {
            AirFnEntry::new(&self.registry, air_fn);
        }

        let output_intermediate_name = format!(
            "{}{}",
            DEDUCTION_INTERMEDIATE_VAR_PREFIX,
            self.registry.get_intermediate_var_index()
        );
        let mut intermediate = O::new(output_intermediate_name.clone());

        #[cfg(test)]
        if self.run {
            let mut air_builder = Self {
                state: State::default(),
                air_body: vec![],
                #[cfg(test)]
                run: self.run,
                registry: self.registry.clone(),
            };
            let output = AirFn::call(air_fn, &mut air_builder, input.clone());
            intermediate = output.let_for_deduction(output_intermediate_name.clone())
        }

        self.air_body.push(AirBodyComponent::LookupCall(LookupCall {
            air_fn_name: air_fn.name(),
            input_arg: input.clone().into(),
            output_name: output_intermediate_name,
        }));

        for felt in intermediate.as_felts() {
            self.deduce(felt);
        }

        self.air_body
            .push(AirBodyComponent::LookupConstraint(LookupConstraint {
                air_fn_name: air_fn.name(),
                input_felts: input.as_felts().into_iter().map(|x| x.clone()).collect(),
                output_felts: intermediate
                    .as_felts()
                    .into_iter()
                    .map(|x| x.clone())
                    .collect(),
            }));

        intermediate
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
    pub air_body: Vec<AirBodyComponent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LookupCall {
    pub air_fn_name: String,
    pub input_arg: GenericAirVar,
    pub output_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LookupConstraint {
    pub air_fn_name: String,

    pub input_felts: Vec<FeltExpr>,
    pub output_felts: Vec<FeltExpr>,
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
    Call(Call),
    LookupCall(LookupCall),
    LookupConstraint(LookupConstraint),
}
