use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;
use serde::Serialize;

use compiled_casm_air::compiled_structs::{
    CompiledAirFn, CompiledAirVar, ConstraintEvalStep, LookupData, TraceGenStep, UseOrYield,
};

use super::air_fn::*;
use super::public_params::*;
use super::state::*;
use super::variables::*;

pub const INTERMEDIATE_VAR_PREFIX: &str = "tmp_";

// AirFnEntry describes everything we know about an Air function.
#[derive(Debug, Clone, Serialize)]
pub struct AirFnEntry {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) inst_def: IndexMap<String, String>,
    pub(crate) input: AirVarImpl,
    pub(crate) input_num_of_felts: usize,
    pub(crate) output: AirVarImpl,
    pub(crate) output_num_of_felts: usize,
    pub(crate) trace_type: TraceType,
    pub(crate) air_body: Vec<AirBodyComponent>,
    pub(crate) state: State,
}

impl AirFnEntry {
    // Compiles the air function entry into a compiled air function.
    pub(crate) fn compile(self) -> CompiledAirFn {
        let (deductions, constraints) = Self::compile_air_body(self.air_body.clone());
        CompiledAirFn {
            name: self.name,
            description: self.description,
            input: self.input.into(),
            output: self.output.into(),
            state_names: self.state.get_state_names(),
            lookup_relation_uses_count: Self::get_lookup_relation_uses_count(deductions.clone()),
            input_num_of_felts: self.input_num_of_felts,
            output_num_of_felts: self.output_num_of_felts,
            constraints,
            deductions,
        }
    }

    // Returns the number of times a lookup relation is used by the air function.
    fn get_lookup_relation_uses_count(deductions: Vec<TraceGenStep>) -> IndexMap<String, usize> {
        let mut lookup_calls = IndexMap::new();
        for deduction in deductions {
            if let TraceGenStep::LookupData(LookupData {
                relation_name,
                use_or_yield,
                ..
            }) = deduction
            {
                if use_or_yield == UseOrYield::Use {
                    *lookup_calls.entry(relation_name).or_insert(0) += 1;
                }
            }
        }
        lookup_calls
    }

    // Transforms the air body of an air function into the compiled air fn format.
    fn compile_air_body(
        air_body: Vec<AirBodyComponent>,
    ) -> (Vec<TraceGenStep>, Vec<ConstraintEvalStep>) {
        let mut constraints = vec![];
        let mut deductions = vec![];

        for component in air_body {
            match component {
                AirBodyComponent::Constraint(constraint, desc) => {
                    constraints.push(ConstraintEvalStep::Constraint(
                        constraint.clone().into(),
                        CompiledAirVar::from(constraint).to_string(),
                        desc,
                    ));
                }
                AirBodyComponent::Assignment {
                    constraint,
                    deduction,
                    desc,
                } => {
                    constraints.push(ConstraintEvalStep::Constraint(
                        constraint.clone().into(),
                        CompiledAirVar::from(constraint).to_string(),
                        desc.clone(),
                    ));
                    deductions.push(TraceGenStep::Deduction(deduction.into()));
                }
                AirBodyComponent::Deduction(deduction, _) => {
                    deductions.push(TraceGenStep::Deduction(deduction.into()));
                }
                AirBodyComponent::Intermediate(name, var, ty) => {
                    if ty.in_constraints {
                        constraints.push(ConstraintEvalStep::Intermediate(
                            name.clone(),
                            var.clone().into(),
                        ));
                    }

                    if ty.in_deductions {
                        deductions.push(TraceGenStep::Intermediate(name, var.into()));
                    }
                }
                AirBodyComponent::Call(f) => {
                    let (new_deductions, new_constraints) = Self::compile_air_body(f.air_body);

                    constraints.push(ConstraintEvalStep::StartBlock(f.air_fn_description.clone()));
                    constraints.extend(new_constraints);
                    constraints.push(ConstraintEvalStep::EndBlock);

                    deductions.push(TraceGenStep::StartBlock(f.air_fn_description));
                    deductions.extend(new_deductions);
                    deductions.push(TraceGenStep::EndBlock);
                }
                AirBodyComponent::LookupCall(call) => {
                    deductions.push(TraceGenStep::LookupCall {
                        fn_name: call.air_fn_name,
                        input: call.input_arg.into(),
                        output_name: call.output_name,
                    });
                }
                AirBodyComponent::LookupData {
                    relation_name,
                    felts,
                    use_or_yield,
                } => {
                    constraints.push(ConstraintEvalStep::LookupData(LookupData {
                        relation_name: relation_name.clone(),
                        felts: felts.clone().into_iter().map(|f| f.into()).collect(),
                        use_or_yield: use_or_yield.clone(),
                    }));
                    deductions.push(TraceGenStep::LookupData(LookupData {
                        relation_name,
                        felts: felts.into_iter().map(|f| f.into()).collect(),
                        use_or_yield,
                    }));
                }
            }
        }

        (deductions, constraints)
    }
}

// AirFnRegistry is created for a specific air function. It keeps all the air function entries
// for the air function and its subroutines.
#[derive(Debug, Clone, Serialize, Default)]
pub struct AirFnRegistry {
    air_fns: Rc<RefCell<IndexMap<String, AirFnEntry>>>,
    #[serde(skip)]
    intermediate_index: Rc<RefCell<usize>>,
    #[serde(skip)]
    pub public_params: PublicParams,
}

impl AirFnRegistry {
    pub fn new_empty() -> Self {
        Self {
            air_fns: Rc::new(RefCell::new(IndexMap::new())),
            intermediate_index: Rc::new(RefCell::new(0)),
            public_params: Default::default(),
        }
    }

    #[cfg(test)]
    pub fn new<I, O>(air_fn: &dyn AirFn<In = I, Out = O>) -> (Self, AirFnEntry)
    where
        I: AirVar,
        O: AirVar,
    {
        let mut registry = Self::new_empty();
        let entry = registry.add_entry(air_fn);
        (registry, entry)
    }

    pub(crate) fn add_entry<I, O>(&mut self, air_fn: &dyn AirFn<In = I, Out = O>) -> AirFnEntry
    where
        I: AirVar,
        O: AirVar,
    {
        if let Some(entry) = self.air_fns.borrow().get(&air_fn.name()) {
            return entry.clone();
        }

        let (air_body, state, input, output) = self.build_air(air_fn);
        let entry = AirFnEntry {
            name: air_fn.name(),
            description: air_fn.description(),
            inst_def: air_fn.inst_def(),
            input: input.clone().into(),
            input_num_of_felts: input.as_felts().len(),
            output: output.clone().into(),
            output_num_of_felts: output.as_felts().len(),
            trace_type: air_fn.trace_type(),
            air_body,
            state,
        };

        self.air_fns
            .borrow_mut()
            .insert(air_fn.name(), entry.clone());

        entry
    }

    // Runs the air function on a given input and returns the resulting state and output.
    #[cfg(test)]
    pub fn run_air<I, O>(&self, air_fn: &dyn AirFn<In = I, Out = O>, input: I) -> (State, O)
    where
        I: AirVar,
        O: AirVar,
    {
        self.run_air_with_row_number(air_fn, input, 0)
    }

    // Runs the air function on a given input in a specific row (relevant if it uses an
    // external column) and returns the resulting state and output.
    #[cfg(test)]
    pub fn run_air_with_row_number<I, O>(
        &self,
        air_fn: &dyn AirFn<In = I, Out = O>,
        input: I,
        row_number: usize,
    ) -> (State, O)
    where
        I: AirVar,
        O: AirVar,
    {
        assert!(self.air_fns.borrow().get(&air_fn.name()).is_some());

        let mut air_builder = AirBuilder {
            state: State::default(),
            air_body: vec![],
            row_number: Some(row_number),
            run: true,
            registry: self.clone(),
        };
        let output = match air_fn.trace_type() {
            TraceType::Inline => air_fn.call(&mut air_builder, input),
            TraceType::Component => air_fn.lookup_call(&mut air_builder, input),
            // For constant AirFns there are no constraints or deductions, so we just return the output.
            TraceType::Const => {
                let output = air_fn.call(&mut air_builder, input);
                assert!(output.is_const(), "Output must be a constant");
                output
            }
            TraceType::Builtin => air_fn.call(&mut air_builder, input),
            TraceType::Opcode => air_fn.lookup_call(&mut air_builder, input),
            TraceType::Memory => air_fn.lookup_call(&mut air_builder, input),
        };

        (air_builder.state, output)
    }

    // Builds the air function on a default input in order to create an air function entry for it.
    fn build_air<I, O>(
        &self,
        air_fn: &dyn AirFn<In = I, Out = O>,
    ) -> (Vec<AirBodyComponent>, State, I, O)
    where
        I: AirVar,
        O: AirVar,
    {
        let input_name = format!("{}_input", air_fn.name().to_lowercase());
        // If input_in_trace is None, we put the input in the trace so air_builder checks don't fail.
        let mut input = I::new(input_name.clone(), true);
        if let Some(input_in_trace) = air_fn.input_in_trace() {
            if !input_in_trace {
                input = I::new(input_name, false);
            }
        }
        let mut air_builder = AirBuilder {
            state: State::default(),
            air_body: vec![],

            // The row number doesn't influence the generated air_body.
            #[cfg(test)]
            row_number: None,
            #[cfg(test)]
            run: false,
            registry: self.clone(),
        };
        let output = match air_fn.trace_type() {
            TraceType::Inline => air_fn.call(&mut air_builder, input.clone()),
            TraceType::Component | TraceType::Opcode => {
                let output = air_fn.lookup_call(&mut air_builder, input.clone());
                // Make sure that the output has no intermediate variables that are not in both
                // constraints and deductions, since the output goes into lookup data (used in
                // trace generation and in constraints evaluation).
                assert!(
                    output.get_intermediate_type().in_constraints && output.get_intermediate_type().in_deductions,
                    "Output must have no intermediate variables that are not in both constraints and deductions",
                );
                output
            }
            // For constant AirFns the value of <output> is meaningless, as we don't
            // output any constraints or deductions. It just has to be of the correct type.
            TraceType::Const => air_fn.call(&mut air_builder, input.clone()),
            TraceType::Builtin => air_fn.call(&mut air_builder, input.clone()),
            TraceType::Memory => air_fn.lookup_call(&mut air_builder, input.clone()),
        };

        // Make sure that the output is a variable or a felt expression.
        let _output_felts = output.as_felts();
        // Make sure that the output is in the state.
        assert!(output.in_state(), "Output must be in the trace");

        (air_builder.air_body, air_builder.state, input, output)
    }

    #[cfg(test)]
    pub fn compile(self) -> IndexMap<String, CompiledAirFn> {
        self.air_fns
            .borrow()
            .iter()
            .enumerate()
            .filter_map(|(i, (name, entry))| {
                if (entry.trace_type == TraceType::Const || entry.trace_type == TraceType::Inline)
                    && (i != self.air_fns.borrow().len() - 1)
                {
                    None
                } else {
                    Some((name.clone(), entry.clone().compile()))
                }
            })
            .collect()
    }

    fn get_intermediate_index(&self) -> usize {
        let mut index = self.intermediate_index.borrow_mut();
        let res = *index;
        *index += 1;
        res
    }

    pub(super) fn get_intermediate_name(&self) -> String {
        let index = self.get_intermediate_index();
        format!("{}{}", INTERMEDIATE_VAR_PREFIX, index)
    }
}
