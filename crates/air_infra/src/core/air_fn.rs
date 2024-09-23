use std::any::type_name;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::air_fn_registry::*;
use super::expressions::felt_expr::*;
use super::memory::*;
use super::state::*;
use super::variables::*;

pub const MAX_NAME_LEN: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceType {
    // Doesn't have its own component in the trace, always inlined into its caller.
    // Can be called only with call.
    Inline,

    // Has its own component in the trace. Each call generates a new row in that component.
    // Can be called only with lookup_call. Generates multiplicity and accumulated sum columns.
    Component,

    // Has its own component in the trace. The trace for this component is pre-filled with rows for
    // all possible inputs by external means. Doesn't generate deductions or constraints.
    // Doesn't generate multiplicity or accumulated sum columns. Has no input, only output.
    // Can be called only with call_external_column.
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

        let mut res = format!("{}_{:?}", name, self.inst_def());
        res = res
            .chars()
            .map(|x| match x {
                ' ' | ':' | '{' | '}' | '\n' | ',' | '[' | ']' => '_',
                _ => x,
            })
            .collect();
        res = res.replace('\"', "");
        while res.contains("__") {
            res = res.replace("__", "_");
        }
        if res.ends_with('_') {
            res.pop();
        }
        if res.len() < MAX_NAME_LEN {
            res
        } else {
            format!("{}_{:x}", name, self.hash())
        }
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

    // For lookup components that their input columns are a const table, return the name of
    // that const table. The const table contains only my input columns and nothing else.
    // Note that the const table might not have a corresponding air function (e.g., "RangeCheck9").
    fn const_input(&self) -> Option<String> {
        None
    }

    fn call(&self, air_builder: &mut AirBuilder, input: Self::In) -> Self::Out;

    fn lookup_call(&self, air_builder: &mut AirBuilder, mut input: Self::In) -> Self::Out {
        assert!(
            self.trace_type() == TraceType::Component,
            "AirFn must be a component"
        );

        if let Some(const_name) = self.const_input() {
            for (i, felt) in input.as_felts_mut().into_iter().enumerate() {
                felt.to_state(i, Some(const_name.clone()));
            }
        } else if !Self::In::is_empty() {
            input = air_builder.let_for_deduction(input);
            for felt in input.as_felts_mut() {
                air_builder.deduce(felt);
            }
        }

        self.call(air_builder, input)
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
    pub(super) row_number: Option<usize>,
    #[cfg(test)]
    pub(super) run: bool,
    pub(super) registry: AirFnRegistry,
}
impl AirBuilder {
    #[cfg(test)]
    pub fn is_run_mode(&self) -> bool {
        self.run
    }

    #[cfg(test)]
    pub fn row_number(&self) -> Option<usize> {
        self.row_number
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

        assert!(
            expr.get_intermediate_type().in_constraints,
            "Constraint contains an intermediate variable that is not in constraints"
        );

        self.air_body.push(AirBodyComponent::Constraint(expr));
    }

    pub fn deduce(&mut self, expr: &mut FeltExpr) -> FeltExpr {
        #[cfg(test)]
        if !self.run {
            // Cannot assert this in run mode, where we might deduce constants.
            assert!(!expr.is_const(), "Cannot deduce a constant");
        }

        assert!(
            expr.get_intermediate_type().in_deductions,
            "Deduction contains an intermediate variable that is not in deductions"
        );

        self.air_body
            .push(AirBodyComponent::Deduction(expr.clone()));
        self.state.add(expr);
        expr.clone()
    }

    pub fn assign(&mut self, expr: &mut FeltExpr) -> FeltExpr {
        #[cfg(test)]
        if !self.run {
            // Cannot assert this in run mode, where we might deduce constants.
            assert!(!expr.is_const(), "Cannot assign a constant");
        }

        #[cfg(test)]
        if self.run {
            // Cannot assert this in build mode, since we don't put the inputs in the state.
            assert!(
                expr.in_state(),
                "The mask of the constraint must be in the trace."
            );
        }

        let intermediate_type = expr.get_intermediate_type();
        assert!(
            intermediate_type.in_deductions && intermediate_type.in_constraints,
            "Assignment contains an intermediate variable that is not in both constraints and deductions"
        );

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
        let name = self.registry.get_intermediate_name();
        let intermediate_type = IntermediateType {
            in_constraints: false,
            in_deductions: true,
        };
        self.air_body.push(AirBodyComponent::Intermediate(
            name.clone(),
            var.clone().into(),
            intermediate_type.clone(),
        ));
        var.let_(name, intermediate_type)
    }

    pub fn let_for_constraint(&mut self, expr: FeltExpr) -> FeltExpr {
        #[cfg(test)]
        if self.run {
            assert!(
                expr.in_state(),
                "The mask of the intermediate variable for constraints must be in the trace."
            );
        }
        let name = self.registry.get_intermediate_name();
        let intermediate_type = IntermediateType {
            in_constraints: true,
            in_deductions: false,
        };
        self.air_body.push(AirBodyComponent::Intermediate(
            name.clone(),
            expr.clone().into(),
            intermediate_type.clone(),
        ));
        expr.let_(name, intermediate_type)
    }

    pub fn let_(&mut self, expr: FeltExpr) -> FeltExpr {
        #[cfg(test)]
        if self.run {
            assert!(
                expr.in_state(),
                "The mask of the intermediate variable for constraints must be in the trace."
            );
        }
        let name = self.registry.get_intermediate_name();
        let intermediate_type = IntermediateType {
            in_constraints: true,
            in_deductions: true,
        };
        self.air_body.push(AirBodyComponent::Intermediate(
            name.clone(),
            expr.clone().into(),
            intermediate_type.clone(),
        ));
        expr.let_(name, intermediate_type)
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

        // Make sure the callee is in the registry
        if self.registry.air_fns.borrow().get(&air_fn.name()).is_none() {
            AirFnEntry::new(&self.registry, air_fn);
        }

        let mut air_builder = Self {
            state: self.state.clone(),
            air_body: vec![],
            #[cfg(test)]
            row_number: self.row_number,
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

    pub fn lookup_call<I, O>(&mut self, air_fn: &dyn AirFn<In = I, Out = O>, input: I) -> O
    where
        I: AirVar,
        O: AirVar,
    {
        assert!(
            air_fn.trace_type() == TraceType::Component,
            "AirFn must be a component"
        );

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

        #[cfg(test)]
        if self.run {
            assert!(
                input.in_state(),
                "The mask of the input to a lookup call must be in the trace."
            );
        }

        let output_name = self.registry.get_intermediate_name();
        let mut output = O::new(output_name.clone());

        #[cfg(test)]
        if self.run {
            let mut air_builder = Self {
                state: State::default(),
                air_body: vec![],
                // When we call a separate component using lookup, we access an arbitrary row in
                // that component (depending on how its rows are sorted). That is, the row number
                // in the callee is not related to the row number in the caller.
                row_number: None,
                run: self.run,
                registry: self.registry.clone(),
            };
            output = air_fn.lookup_call(&mut air_builder, input.clone());
        }

        self.air_body.push(AirBodyComponent::LookupCall(LookupCall {
            air_fn_name: air_fn.name(),
            input_arg: input.clone().into(),
            output_name: if O::is_empty() {
                None
            } else {
                Some(output_name.clone())
            },
        }));

        if !O::is_empty() {
            output = output.let_(
                output_name.clone(),
                IntermediateType {
                    in_constraints: false,
                    in_deductions: true,
                },
            );

            for felt in output.as_felts_mut() {
                self.deduce(felt);
            }
        }

        self.air_body
            .push(AirBodyComponent::LookupConstraint(LookupConstraint {
                air_fn_name: air_fn.name(),
                input_felts: input.as_felts(),
                output_felts: output.as_felts(),
            }));

        output
    }

    #[allow(unused_variables)]
    // Reads the value from the memory, creates an intermediate variable for the value, and returns
    // it. Does not add any constraints or deductions.
    pub fn mem_read_unverified<K, V>(&mut self, memory: &Memory<K, V>, key: &K) -> V
    where
        K: AirVar + Default,
        V: AirVar + Default,
    {
        // Make sure the memory is in the registry
        if self
            .registry
            .air_fns
            .borrow()
            .get(&(memory.name()))
            .is_none()
        {
            AirFnEntry::new(&self.registry, memory);
        }

        let value_name = self.registry.get_intermediate_name();

        self.air_body.push(AirBodyComponent::LookupCall(LookupCall {
            air_fn_name: memory.name(),
            input_arg: key.clone().into(),
            output_name: Some(value_name.clone()),
        }));

        #[allow(unused_mut)]
        let mut value = V::new(value_name.clone());

        #[cfg(test)]
        if self.run {
            let mut air_builder = Self {
                state: State::default(),
                air_body: vec![],
                // This is None for the same reason as in lookup_call.
                row_number: None,
                run: self.run,
                registry: self.registry.clone(),
            };
            value = memory.lookup_call(&mut air_builder, key.clone());
        }

        value.let_(
            value_name,
            IntermediateType {
                in_constraints: false,
                in_deductions: true,
            },
        )
    }

    #[allow(unused_variables)]
    // Assumes the key and value are in the state (of the caller). Adds a lookup constraint.
    // Writes the value to the memory in run and cairo run modes.
    pub fn mem_verify<K, V>(&mut self, memory: &Memory<K, V>, key: &K, value: V)
    where
        K: AirVar + Default,
        V: AirVar + Default,
    {
        // Make sure the memory is in the registry
        if self
            .registry
            .air_fns
            .borrow()
            .get(&(memory.name()))
            .is_none()
        {
            AirFnEntry::new(&self.registry, memory);
        }

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

    #[allow(unused_variables)]
    pub fn call_external_column<O>(&mut self, air_fn: &dyn AirFn<In = (), Out = O>) -> O
    where
        O: AirVar,
    {
        assert!(
            air_fn.trace_type() == TraceType::Const,
            "External columns must be constant"
        );

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

        #[cfg(test)]
        if self.run {
            let mut air_builder = Self {
                state: State::default(),
                air_body: vec![],
                #[cfg(test)]
                row_number: self.row_number,
                #[cfg(test)]
                run: self.run,
                registry: self.registry.clone(),
            };
            return air_fn.call(&mut air_builder, ());
        }

        let mut output = O::new("".to_string());
        for (i, felt) in output.as_felts_mut().into_iter().enumerate() {
            felt.to_state(i, Some(air_fn.name()));
        }
        output
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
    // None if there is no output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_name: Option<String>,
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
    Intermediate(String, AirVarImpl, IntermediateType),
    Call(Call),
    LookupCall(LookupCall),
    LookupConstraint(LookupConstraint),
}
