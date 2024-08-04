use std::any::type_name;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::air_fn_registry::*;
#[cfg(test)]
use super::expressions::expr::*;
use super::expressions::felt_expr::*;
use super::memory::*;
use super::state::*;
use super::variables::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceType {
    // Doesn't have its own component in the trace, always inlined into its caller.
    Inline,

    // Has its own component in the trace. Each call generates a new row in that component.
    Component,

    // Has its own component in the trace. The trace for this component is pre-filled
    // with rows for all possible inputs by external means. Doesn't generate deductions
    // or constraints.
    Const,
}

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
            .find('<')
            .map(|i| name[..i].to_string())
            .unwrap_or(name);
        name = name
            .rfind("::")
            .map(|i| name[i + 2..].to_string())
            .unwrap_or(name);

        format!("{}_{:x}", name, self.hash())
    }

    fn hash(&self) -> u64 {
        let name = format!("{}{:?}", type_name::<Self>(), self.inst_def());
        let mut s = DefaultHasher::new();
        name.hash(&mut s);
        s.finish()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Inline
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        IndexMap::new()
    }

    // Assumes the input was written to the trace
    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out;

    fn lookup_call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out {
        assert!(
            self.trace_type() == TraceType::Component,
            "AirFn must be a component"
        );

        let mut input_in_state = air_builder.let_for_deduction(input);
        for felt in input_in_state.as_felts_mut() {
            air_builder.deduce(felt);
        }

        self.call(air_builder, input_in_state)
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
    #[cfg(test)]
    pub(super) internal_component: bool,
    pub(super) registry: AirFnRegistry,
}
impl AirBuilder {
    #[cfg(test)]
    pub fn is_run_mode(&self) -> bool {
        self.run
    }

    pub fn constrain(&mut self, expr: FeltExpr) {
        #[cfg(test)]
        if self.run {
            // Cannot assert this in build mode, since we don't put the inputs in the state.
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
        #[cfg(test)]
        if !self.run || !self.internal_component {
            // Cannot assert this in run mode on internal component, where we might deduce constants.
            assert!(!expr.is_const());
        }
        self.air_body
            .push(AirBodyComponent::Deduction(expr.clone()));
        self.state.add(expr);
        expr.clone()
    }

    pub fn assign(&mut self, expr: &mut FeltExpr) -> FeltExpr {
        #[cfg(test)]
        if !self.run || !self.internal_component {
            // Cannot assert this in run mode on internal component, where we might deduce constants.
            assert!(!expr.is_const());
        }

        #[cfg(test)]
        if self.run {
            // Cannot assert this in build mode, since we don't put the inputs in the state.
            assert!(
                expr.in_state(),
                "The mask of the constraint must be in the trace."
            );
        }

        let before = expr.clone();
        self.state.add(expr);

        let constraint = expr.clone() - before.clone();
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
        let name = self.registry.get_deduction_intermediate_var_name();
        self.air_body.push(AirBodyComponent::DeductionIntermediate(
            name.clone(),
            var.clone().into(),
        ));
        var.let_for_deduction(name)
    }

    pub fn let_for_constraint(&mut self, expr: FeltExpr) -> FeltExpr {
        #[cfg(test)]
        if self.run {
            assert!(
                expr.in_state(),
                "The mask of the intermediate variable for constraints must be in the trace."
            );
        }
        let name = self.registry.get_constraint_intermediate_var_name();
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
        // It is technically possible to inline-call an AirFn that has TraceType::Component
        // (by adding the constraints and deductions of that component to the current
        // component), but doing so is usually a mistake as it is less efficient than
        // doing a lookup. Therefore we only allow calling AirFns with TraceType::Inline
        assert!(
            air_fn.trace_type() == TraceType::Inline,
            "AirFn must be inline"
        );

        #[cfg(test)]
        if self.run {
            assert!(input.in_state(), "Input must be in the trace");
        }

        // Make sure the callee is in the registry
        if self.registry.air_fns.borrow().get(&air_fn.name()).is_none() {
            AirFnEntry::new(&self.registry, air_fn);
        }

        let mut air_builder = Self {
            state: self.state.clone(),
            air_body: vec![],
            #[cfg(test)]
            run: self.run,
            #[cfg(test)]
            internal_component: self.internal_component,
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

    pub fn lookup_call<I, O>(&mut self, air_fn: &dyn AirFn<In = I, Out = O>, input: I) -> O
    where
        I: AirVar,
        O: AirVar,
    {
        match air_fn.trace_type() {
            TraceType::Inline => {
                panic!("Lookup call cannot be used with an AirFn that is not a separate component")
            }
            TraceType::Component | TraceType::Const => (),
        }

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

        let output_intermediate_name = self.registry.get_deduction_intermediate_var_name();
        let mut intermediate = O::new(output_intermediate_name.clone());

        #[cfg(test)]
        if self.run {
            let mut air_builder = Self {
                state: State::default(),
                air_body: vec![],
                #[cfg(test)]
                run: self.run,
                #[cfg(test)]
                internal_component: true,
                registry: self.registry.clone(),
            };
            let output = match air_fn.trace_type() {
                // For const components, use call() to compute the output
                TraceType::Const => air_fn.call(&mut air_builder, input.clone()),

                // For regular components, call() is not supposed to be called directly but
                // only through lookup_call().
                TraceType::Component => air_fn.lookup_call(&mut air_builder, input.clone()),

                _ => panic!(),
            };

            intermediate = output.let_for_deduction(output_intermediate_name.clone())
        }

        self.air_body.push(AirBodyComponent::LookupCall(LookupCall {
            air_fn_name: air_fn.name(),
            input_arg: input.clone().into(),
            output_name: output_intermediate_name,
        }));

        for felt in intermediate.as_felts_mut() {
            self.deduce(felt);
        }

        self.air_body
            .push(AirBodyComponent::LookupConstraint(LookupConstraint {
                air_fn_name: air_fn.name(),
                input_felts: input.as_felts(),
                output_felts: intermediate.as_felts(),
            }));

        intermediate
    }

    #[allow(unused_variables)]
    // Reads the value from the memory, creates an intermediate variable for the value, and returns
    // it. Does not add any constraints or deductions.
    pub fn get_from_memory<K, V>(&mut self, memory: &Memory<K, V>, key: &K) -> V
    where
        K: AirVar,
        V: AirVar + Default,
    {
        let value_name = self.registry.get_deduction_intermediate_var_name();

        self.air_body.push(AirBodyComponent::LookupCall(LookupCall {
            air_fn_name: memory.name(),
            input_arg: key.clone().into(),
            output_name: value_name.clone(),
        }));

        #[allow(unused_mut)]
        let mut value = V::new(value_name.clone());

        #[cfg(test)]
        if self.run {
            value = memory.get(key).unwrap();
            value = value.let_for_deduction(value_name);
        }

        value
    }

    #[allow(unused_variables)]
    // Assumes the key and value are in the state (of the caller). Adds a lookup constraint.
    // Writes the value to the memory in run and cairo run modes.
    pub fn set_in_memory<K, V>(&mut self, memory: &Memory<K, V>, key: K, value: V)
    where
        K: AirVar,
        V: AirVar + Default,
    {
        #[cfg(test)]
        if self.run {
            assert!(key.in_state(), "The key must be in the trace.");
            assert!(value.in_state(), "The value must be in the trace.");

            memory.set(key.clone(), value.clone());
        }

        if self
            .registry
            .air_fns
            .borrow()
            .get(&(memory.name()))
            .is_none()
        {
            AirFnEntry::new(&self.registry, memory);
        }

        self.air_body
            .push(AirBodyComponent::LookupConstraint(LookupConstraint {
                air_fn_name: memory.name(),
                input_felts: key.as_felts(),
                output_felts: value.as_felts(),
            }));
    }
}

// A Call is an air_body component that represents a call to another air function.
// It contains the name of the air function, the input argument, the output of the call, the state
// after the call, and the air_body of the called function.
#[derive(Clone, Debug, Serialize)]
pub struct Call {
    pub air_fn_name: String,
    pub input_arg: AirVarImpl,
    pub output: AirVarImpl,
    #[serde(skip)]
    pub air_body: Vec<AirBodyComponent>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LookupCall {
    pub air_fn_name: String,
    pub input_arg: AirVarImpl,
    pub output_name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LookupConstraint {
    pub air_fn_name: String,

    pub input_felts: Vec<FeltExpr>,
    pub output_felts: Vec<FeltExpr>,
}

// Each air function has an air_body, which is a vector of AirBodyComponent.
// These are the components of the air function.
#[derive(Clone, Debug, Serialize)]
pub enum AirBodyComponent {
    Constraint(FeltExpr),
    Deduction(FeltExpr),
    // An assignment is a constraint and a deduction referring to the same trace cell.
    // For example, when copying a value from one trace cell to another.
    Assignment {
        constraint: FeltExpr,
        deduction: FeltExpr,
    },
    DeductionIntermediate(String, AirVarImpl),
    ConstraintIntermediate(String, FeltExpr),
    Call(Call),
    LookupCall(LookupCall),
    LookupConstraint(LookupConstraint),
}
