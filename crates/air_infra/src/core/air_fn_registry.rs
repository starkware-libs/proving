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
use crate::core::constraint_connectedness_test::assert_constraint_graph_connected;

// AirFnEntry describes everything we know about an Air function.
#[derive(Debug, Clone)]
pub struct AirFnEntry {
    pub name: String,
    pub relation_names: Vec<String>,
    // For constant-size component, the log_2 of the number of rows.
    pub log_height: Option<u32>,
    pub description: String,
    pub inst_def: serde_json::Value,
    pub ext_input: Option<AirVarImpl>,
    pub input: Option<AirVarImpl>,
    pub input_const_columns: Vec<String>,
    pub joined_input: AirVarImpl,
    pub input_expr_descriptions: Option<Vec<Option<String>>>,
    // <true> for felts in <joined_input> that participate in constraints, <false> otherwise.
    pub input_limbs_mask: Vec<bool>,
    pub output: AirVarImpl,
    pub output_expr_descriptions: Option<Vec<Option<String>>>,
    // <false> for felts in <output> that are directly in state, <true> otherwise.
    pub output_limbs_mask: Vec<bool>,
    pub trace_type: TraceType,
    pub padding_type: PaddingType,
    pub air_body: AirBody,
    pub state: State,
}

impl AirFnEntry {
    pub fn new<E, I, O>(
        air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>,
        air_body: AirBody,
        state: State,
        ext_input: E::T,
        input: I,
        output: O,
    ) -> Self
    where
        E: ExtTable,
        I: AirVar,
        O: AirVar,
    {
        let ext_input_option = (!E::T::is_empty()).then(|| ext_input.clone().into());
        let input_option = (!I::is_empty()).then(|| input.clone().into());
        let joined_input = AirFnEntry::join_inputs(ext_input_option.clone(), input_option.clone());
        let mut constraint_intermediates = air_body.get_used_constraint_intermediates();
        constraint_intermediates.extend(
            output
                .as_felts()
                .into_iter()
                .flat_map(|f| f.get_used_constraint_intermediates()),
        );
        let input_name = AirFnEntry::input_name(&air_fn.name());
        let input_limbs_mask = (0..joined_input.as_felts().len())
            .map(|i| {
                let name =
                    joined_input.get_limb_name(&input_name, i, &air_fn.input_expr_descriptions());
                constraint_intermediates.contains(&name)
            })
            .collect();
        let output_limbs_mask = output
            .as_felts()
            .into_iter()
            .map(|f| (!f.is_directly_in_state()))
            .collect();

        Self {
            name: air_fn.name(),
            relation_names: air_fn.relation_names(),
            description: air_fn.description(),
            log_height: E::log_size(),
            inst_def: air_fn.inst_def(),
            ext_input: ext_input_option,
            input: input_option,
            input_const_columns: E::column_ids(),
            joined_input,
            input_expr_descriptions: air_fn.input_expr_descriptions(),
            input_limbs_mask,
            output: output.into(),
            output_expr_descriptions: air_fn.output_expr_descriptions(),
            output_limbs_mask,
            trace_type: air_fn.trace_type(),
            padding_type: air_fn.padding_type(),
            air_body,
            state,
        }
    }

    // Compiles the air function entry into a compiled air function.
    pub fn compile(self, called_fns: &Ref<'_, IndexMap<String, AirFnEntry>>) -> CompiledAirFn {
        let inline_calls = self
            .air_body
            .get_inline_calls()
            .iter()
            .map(|n| {
                let ab = &called_fns
                    .get(n)
                    .unwrap_or_else(|| panic!("Cannot find called air function {n}"))
                    .air_body;
                (
                    n.clone(),
                    (
                        ab.get_constraint_lookups().clone(),
                        ab.get_public_params().clone(),
                        ab.get_external_states().clone(),
                    ),
                )
            })
            .collect();
        let sub_components = self
            .air_body
            .get_sub_components()
            .into_iter()
            .map(|name| {
                called_fns
                    .get(name.as_str())
                    .as_ref()
                    .map(|entry| (name, entry.padding_type))
                    .expect("Cannot find sub-component air function")
            })
            .collect::<IndexMap<_, _>>();
        let relation_size =
            if self.trace_type == TraceType::Opcode || self.trace_type == TraceType::ChainRound {
                Some(self.joined_input.as_felts().len())
            } else if self.relation_names.is_empty() {
                None
            } else {
                Some(self.joined_input.as_felts().len() + self.output.as_felts().len())
            };
        let n_inputs_added_per_relation = self
            .air_body
            .get_n_inputs_added_per_relation()
            .iter()
            .map(|(relation_name, (component_name, max_inputs))| {
                let relation_index = called_fns
                    .get(component_name.as_str())
                    .as_ref()
                    .expect("Cannot find called component air function")
                    .relation_names
                    .iter()
                    .position(|r| r == relation_name)
                    .expect("Relation name not found in air function relation names");
                (
                    relation_name.clone(),
                    (component_name.clone(), relation_index, *max_inputs),
                )
            })
            .collect::<IndexMap<_, _>>();

        CompiledAirFn {
            name: self.name.clone(),
            relation_names: self.relation_names.clone(),
            relation_size,
            log_height: self.log_height,
            description: self.description.clone(),
            instance_definition: serde_json::to_string(&self.inst_def)
                .expect("Failed to serialize"),
            r#type: self.trace_type,
            padding_type: self.padding_type,
            prover_input: (
                Self::input_name(&self.name),
                self.joined_input.prover_type(),
                self.joined_input.packed_prover_type(),
            ),
            verifier_input_limbs: self.input_limb_names(),
            input_const_columns: self.input_const_columns.clone(),
            prover_output: (
                self.output.clone().compile(CompileFor::Deductions),
                Self::output_name(&self.name),
                self.output.prover_type(),
                self.output.packed_prover_type(),
            ),
            verifier_output: (
                self.filter_output_limbs(self.output.clone())
                    .compile(CompileFor::Constraints),
                self.output_limb_names(Self::output_name(&self.name)),
            ),
            state_names: self.state.get_state_names(),
            constraint_lookups: self.air_body.get_constraint_lookups(),
            sub_components,
            n_inputs_added_per_relation,
            inline_calls,
            constraints: self.air_body.compile_for_constraints(),
            deductions: self.air_body.compile_for_deductions(),
            public_params: self.air_body.get_public_params(),
            external_states: self.air_body.get_external_states(),
        }
    }

    pub fn join_inputs(ext_input: Option<AirVarImpl>, input: Option<AirVarImpl>) -> AirVarImpl {
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

    // Given an output of the air function, returns an array of the felts of the output that are in
    // the mask.
    pub fn filter_output_limbs(&self, output: AirVarImpl) -> AirVarImpl {
        AirVarImpl::Array(
            self.output_limbs_mask
                .iter()
                .zip(output.as_felts())
                .filter_map(|(b, f)| b.then(|| f.into()))
                .collect(),
        )
    }

    // Given an input to the air function, returns an array of the felts of the input that are in
    // the mask.
    pub fn filter_input_limbs(&self, input: AirVarImpl) -> AirVarImpl {
        AirVarImpl::Array(
            self.input_limbs_mask
                .iter()
                .zip(input.as_felts())
                .filter_map(|(b, f)| b.then(|| f.into()))
                .collect(),
        )
    }

    pub fn input_limbs_type(&self) -> String {
        self.filter_input_limbs(self.joined_input.clone())
            .prover_type()
    }

    // Given the name of the output intermediate variable, uses the <output_expr_descriptions> and
    // returns the names of the limbs of the output (the felts that are in the mask).
    pub fn output_limb_names(&self, name: String) -> Vec<String> {
        self.output_limbs_mask
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .map(|(i, _)| {
                self.output
                    .get_limb_name(&name, i, &self.output_expr_descriptions)
            })
            .collect::<Vec<_>>()
    }

    // Uses the name of the input intermediate variable (see <input_name>) and the
    // <input_expr_descriptions>, and returns the names of the limbs of the input (the felts that
    // are in the mask).
    pub fn input_limb_names(&self) -> Vec<String> {
        let name = Self::input_name(&self.name);
        self.input_limbs_mask
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .map(|(i, _)| {
                self.joined_input
                    .get_limb_name(&name, i, &self.input_expr_descriptions)
            })
            .collect::<Vec<_>>()
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
        let entry = AirFnEntry::new(air_fn, air_body, state, ext_input, input, output);

        // Don't test inline AirFns. It is OK to have an inline AirFn with a disconnected
        // constraint graph. For example, an AirFn that receives a Felt252 as 28 M31s and checks
        // that each of the M31s contains 9 bits.
        if entry.trace_type != TraceType::Inline {
            assert_constraint_graph_connected(&entry);
        }

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
            .map(|(name, entry)| (name.clone(), entry.clone().compile(&self.air_fns.borrow())))
            .collect()
    }
}
