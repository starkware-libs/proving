use std::cell::{Ref, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use compiled_casm_air::compiled_structs::{CompiledAirFn, PaddingType, TraceType};
use compiled_casm_air::utils::{INPUT_VAR_SUFFIX, OUTPUT_VAR_SUFFIX};
use indexmap::IndexMap;

use super::air_body::*;
use super::air_fn::*;
use super::public_params::*;
use super::state::*;
use super::variables::*;

// AirFnEntry describes everything we know about an Air function.
#[derive(Debug, Clone)]
pub struct AirFnEntry {
    pub name: String,
    pub relation_name: Option<String>,
    pub description: String,
    pub inst_def: serde_json::Value,
    pub ext_input: Option<AirVarImpl>,
    pub input: Option<AirVarImpl>,
    pub input_expr_descriptions: Option<Vec<Option<String>>>,
    pub output: AirVarImpl,
    pub output_expr_descriptions: Option<Vec<Option<String>>>,
    pub trace_type: TraceType,
    pub air_body: AirBody,
    pub state: State,
}

impl AirFnEntry {
    // Compiles the air function entry into a compiled air function.
    pub fn compile(self, called_fns: Ref<'_, IndexMap<String, AirFnEntry>>) -> CompiledAirFn {
        let padding_type = match self.trace_type {
            TraceType::Builtin | TraceType::Const | TraceType::Inline => PaddingType::None,
            TraceType::Opcode | TraceType::ChainRound => PaddingType::Enabler,
            TraceType::Memory => PaddingType::Multiplicity,
            TraceType::Component if self.name == "verify_instruction" => PaddingType::Multiplicity,
            TraceType::Component if self.ext_input.is_some() => PaddingType::Multiplicity,
            _ => PaddingType::Enabler,
        };
        let input = Self::generate_input(self.ext_input.clone(), self.input.clone());
        let inline_calls = self
            .air_body
            .get_inline_calls()
            .into_iter()
            .map(|n| {
                let ab = &called_fns
                    .get(&n)
                    .expect("Cannot find called air function")
                    .air_body;
                (
                    n.clone(),
                    (
                        ab.get_lookup_names().clone(),
                        ab.get_public_params().clone(),
                        ab.get_external_states().clone(),
                    ),
                )
            })
            .collect();
        let relation_size =
            if self.trace_type == TraceType::Opcode || self.trace_type == TraceType::ChainRound {
                Some(input.as_felts().len())
            } else {
                self.relation_name
                    .clone()
                    .map(|_| input.as_felts().len() + self.output.as_felts().len())
            };

        CompiledAirFn {
            name: self.name.clone(),
            relation_name: self.relation_name.clone(),
            relation_size,
            description: self.description.clone(),
            instance_definition: serde_json::to_string(&self.inst_def)
                .expect("Failed to serialize"),
            r#type: self.trace_type,
            padding_type,
            prover_input: (
                Self::input_name(&self.name),
                input.prover_type(),
                input.packed_prover_type(),
            ),
            verifier_input: (self.input_verifier_name(), self.input_verifier_type()),
            prover_output: (
                self.output.clone().compile(CompileFor::Deductions),
                Self::output_name(&self.name),
                self.output.prover_type(),
                self.output.packed_prover_type(),
            ),
            verifier_output: (
                self.output_as_limbs(self.output.clone())
                    .compile(CompileFor::Constraints),
                self.output_verifier_name(Self::output_name(&self.name)),
                self.output_verifier_type(),
            ),
            state_names: self.state.get_state_names(),
            lookup_names: self
                .air_body
                .get_lookup_names()
                .into_iter()
                .map(|(name, _)| name)
                .collect(),
            inline_calls,
            constraints: self.air_body.compile_for_constraints(),
            deductions: self.air_body.compile_for_deductions(),
            public_params: self.air_body.get_public_params(),
            external_states: self.air_body.get_external_states(),
        }
    }

    pub fn generate_input(ext_input: Option<AirVarImpl>, input: Option<AirVarImpl>) -> AirVarImpl {
        match (ext_input, input) {
            (Some(ext_input), None) => ext_input,
            (None, Some(input)) => input,
            (Some(ext_input), Some(input)) => AirVarImpl::Tuple(vec![ext_input, input]),
            (None, None) => AirVarImpl::Tuple(vec![]),
        }
    }

    pub fn output_name(air_fn_name: &String) -> String {
        format!("{}_{}", air_fn_name, OUTPUT_VAR_SUFFIX)
    }

    pub fn input_name(air_fn_name: &String) -> String {
        format!("{}_{}", air_fn_name, INPUT_VAR_SUFFIX)
    }

    pub fn output_as_limbs(&self, output: AirVarImpl) -> AirVarImpl {
        AirVarImpl::Array(
            self.get_output_optional_limbs(output)
                .into_iter()
                .flatten()
                .collect(),
        )
    }

    pub fn input_as_limbs(&self, input: AirVarImpl) -> AirVarImpl {
        AirVarImpl::Array(
            self.get_input_optional_limbs(input)
                .into_iter()
                .flatten()
                .collect(),
        )
    }

    pub fn output_verifier_type(&self) -> String {
        self.output_as_limbs(self.output.clone()).prover_type()
    }

    pub fn input_verifier_type(&self) -> String {
        let input = Self::generate_input(self.ext_input.clone(), self.input.clone());
        self.input_as_limbs(input).prover_type()
    }

    pub fn output_verifier_name(&self, name: String) -> String {
        let optional_limbs = self.get_output_optional_limbs(self.output.clone());
        if optional_limbs.iter().all(|v| v.is_none()) {
            return "()".to_string();
        }

        format!(
            "[{}]",
            optional_limbs
                .into_iter()
                .enumerate()
                .filter_map(|(i, v)| v.map(|_| self.output.get_limb_name(
                    &name,
                    i,
                    &self.output_expr_descriptions
                )))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub fn input_verifier_name(&self) -> String {
        let input = Self::generate_input(self.ext_input.clone(), self.input.clone());
        let optional_limbs = self.get_input_optional_limbs(input.clone());
        let name = Self::input_name(&self.name);

        if optional_limbs.iter().all(|v| v.is_none()) {
            return "()".to_string();
        }

        format!(
            "[{}]",
            optional_limbs
                .into_iter()
                .enumerate()
                .filter_map(|(i, v)| v.map(|_| input.get_limb_name(
                    &name,
                    i,
                    &self.input_expr_descriptions
                )))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn get_output_optional_limbs(&self, output: AirVarImpl) -> Vec<Option<AirVarImpl>> {
        output
            .as_felts()
            .into_iter()
            .zip(self.output.as_felts())
            .map(|(f, entry_f)| (!entry_f.is_directly_in_state()).then_some(f.into()))
            .collect()
    }

    fn get_input_optional_limbs(&self, input: AirVarImpl) -> Vec<Option<AirVarImpl>> {
        let input_name = Self::input_name(&self.name);
        let mut constraint_intermediates = self.air_body.get_constraint_intermediates();
        constraint_intermediates.extend(
            self.output
                .as_felts()
                .into_iter()
                .flat_map(|f| f.get_constraint_intermediates()),
        );
        let entry_input = Self::generate_input(self.ext_input.clone(), self.input.clone());

        input
            .as_felts()
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                let name = entry_input.get_limb_name(&input_name, i, &self.input_expr_descriptions);
                constraint_intermediates.contains(&name).then_some(f.into())
            })
            .collect()
    }
}

// AirFnRegistry is created for a specific air function. It keeps all the air function entries
// for the air function and its subroutines.
#[derive(Debug, Clone, Default)]
pub struct AirFnRegistry {
    pub air_fns: Rc<RefCell<IndexMap<String, AirFnEntry>>>,
    pub air_fn_ids: Rc<RefCell<HashSet<String>>>,
    pub public_params: PublicParams,
}

impl AirFnRegistry {
    pub fn new_empty() -> Self {
        Self {
            air_fns: Rc::new(RefCell::new(IndexMap::new())),
            air_fn_ids: Rc::new(RefCell::new(HashSet::new())),
            public_params: Default::default(),
        }
    }

    #[cfg(test)]
    pub fn new<E, I, O>(air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>) -> (Self, AirFnEntry)
    where
        E: ExtTable,
        I: AirVar,
        O: AirVar,
    {
        let mut registry = Self::new_empty();
        let entry = registry.add_entry(air_fn);
        (registry, entry)
    }

    pub fn add_entry<E, I, O>(
        &mut self,
        air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>,
    ) -> AirFnEntry
    where
        E: ExtTable,
        I: AirVar,
        O: AirVar,
    {
        if let Some(entry) = self.air_fns.borrow().get(&air_fn.name()) {
            return entry.clone();
        }

        if !E::T::is_empty() {
            let ext_input_air_fn = ExtTableAirFn::<E>::default();
            self.add_entry(&ext_input_air_fn);
        }

        let air_fn_id = air_fn.hash();
        assert!(
            !self.air_fn_ids.borrow().contains(&air_fn_id),
            "Air function with the same hash already exists"
        );
        self.air_fn_ids.borrow_mut().insert(air_fn_id.clone());

        let (air_body, state, ext_input, input, output) = self.build_air(air_fn, air_fn_id);
        let ext_input_option = (!E::T::is_empty()).then(|| ext_input.clone().into());
        let input_option = (!I::is_empty()).then(|| input.clone().into());

        let entry = AirFnEntry {
            name: air_fn.name(),
            relation_name: air_fn.relation_name(),
            description: air_fn.description(),
            inst_def: air_fn.inst_def(),
            ext_input: ext_input_option,
            input: input_option,
            input_expr_descriptions: air_fn.input_expr_descriptions(),
            output: output.into(),
            output_expr_descriptions: air_fn.output_expr_descriptions(),
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
    pub fn run_air<E, I, O>(
        &self,
        air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>,
        ext_input: E::T,
        input: I,
    ) -> (State, O)
    where
        E: ExtTable,
        I: AirVar,
        O: AirVar,
    {
        self.run_air_with_row_number(air_fn, ext_input, input, 0)
    }

    // Runs the air function on a given input in a specific row (relevant if it uses an
    // external column) and returns the resulting state and output.
    #[cfg(test)]
    pub fn run_air_with_row_number<E, I, O>(
        &self,
        air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>,
        ext_input: E::T,
        input: I,
        row_number: usize,
    ) -> (State, O)
    where
        E: ExtTable,
        I: AirVar,
        O: AirVar,
    {
        assert!(self.air_fns.borrow().get(&air_fn.name()).is_some());

        let mut air_builder = AirBuilder {
            component_context: Default::default(),
            air_body: AirBody::default(),
            row_number: Some(row_number),
            run: true,
            registry: self.clone(),
            intermediate_id: Rc::new(RefCell::new(("".to_string(), 0))),
        };
        let output = match air_fn.trace_type() {
            TraceType::Inline | TraceType::Builtin => {
                air_fn.call(&mut air_builder, ext_input, input)
            }
            TraceType::Component
            | TraceType::ChainRound
            | TraceType::Memory
            | TraceType::Opcode => air_fn.lookup_call(&mut air_builder, ext_input, input),
            // For constant AirFns there are no constraints or deductions, so we just return the
            // output.
            TraceType::Const => {
                let output = air_fn.call(&mut air_builder, ext_input, input);
                assert!(
                    output.clone().into().is_const(),
                    "Output must be a constant"
                );
                output
            }
        };

        let state = air_builder.component_context.state().clone();
        (state, output)
    }

    // Builds the air function on a default input in order to create an air function entry for it.
    fn build_air<E, I, O>(
        &self,
        air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>,
        air_fn_id: String,
    ) -> (AirBody, State, E::T, I, O)
    where
        E: ExtTable,
        I: AirVar,
        O: AirVar,
    {
        let ext_input = E::new();
        let input_name = AirFnEntry::input_name(&air_fn.name());
        let mut input = I::new(input_name.clone(), Some(1));
        input = input
            .rec_let(input_name, air_fn.input_expr_descriptions())
            .0;

        let mut air_builder = AirBuilder {
            component_context: Default::default(),
            air_body: AirBody::default(),

            // The row number doesn't influence the generated air_body.
            #[cfg(test)]
            row_number: None,
            #[cfg(test)]
            run: false,
            registry: self.clone(),
            intermediate_id: Rc::new(RefCell::new((air_fn_id, 0))),
        };

        let output = match air_fn.trace_type() {
            // For constant AirFns the value of <output> is meaningless, as we don't
            // output any constraints or deductions. It just has to be of the correct type.
            TraceType::Inline | TraceType::Builtin | TraceType::Const => {
                air_fn.call(&mut air_builder, ext_input.clone(), input.clone())
            }
            TraceType::Component
            | TraceType::ChainRound
            | TraceType::Opcode
            | TraceType::Memory => {
                let output = air_fn.lookup_call(&mut air_builder, ext_input.clone(), input.clone());
                // Make sure that all intermediate variables in the output are visible in the
                // trace generation code, since this code returns the output.
                assert!(
                    output.clone().into().visibility().in_deductions,
                    "Output must have no intermediate variables that are not in deductions",
                );
                output
            }
        };

        // Make sure that the output is a variable or a felt expression.
        let output_felts = output.as_felts();
        // Make sure that the output is in the state.
        assert!(
            output_felts.iter().all(|felt| felt.in_state()),
            "Output felts must be in the trace"
        );

        let state = air_builder.component_context.state().clone();
        (air_builder.air_body, state, ext_input, input, output)
    }

    pub fn compile(&self) -> IndexMap<String, CompiledAirFn> {
        self.air_fns
            .borrow()
            .iter()
            .map(|(name, entry)| (name.clone(), entry.clone().compile(self.air_fns.borrow())))
            .collect()
    }
}
