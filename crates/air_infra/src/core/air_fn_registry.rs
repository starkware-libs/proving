use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use air_common::utils::random_m31;
use air_common::{PaddingType, TraceType, UseOrYield};
use air_compile::compiled_structs::CompiledAirFn;
use convert_case::{Case, Casing};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use stwo_cairo_common::prover_types::cpu::{M31, PRIME};

use super::air_body::*;
use super::air_fn::*;
use super::public_params::*;
use super::state::*;
use super::variables::*;
use crate::const_expr;
use crate::core::constraint_connectedness_test::assert_constraint_graph_connected;
use crate::core::expressions::felt_expr::{FeltExpr, ValueInfo};

pub const LOOKUPS_PER_BATCH: usize = 2;
pub const TRACE_COLUMNS_PER_LOOKUP_BATCH: usize = 4;
// Assumes blowup factor at most 4
pub const MAX_ROWS_PER_TABLE: usize = 2_usize.pow(28);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AirFnStat {
    pub trace_type: TraceType,
    // For constant-size component, the log_2 of the number of rows.
    pub log_height: Option<u32>,
    // The number of cells in each row, excluding interaction columns.
    pub num_state_cols: usize,
    // How many use lookups each row does. A map of relation name -> number of lookups to this
    // relation.
    pub use_lookup_cols: IndexMap<String, usize>,
    // How many yield lookups each row does. A map of relation name -> number of lookups to this
    // relation.
    pub yield_lookup_cols: IndexMap<String, usize>,
    // How many rows each row in this component adds to other components, keyed by relation name.
    // This is higher than `use_lookup_cols` for calls to chain round components. For example,
    // blake_compress_opcode adds 10 rows to blake_round, but does only one use lookup and one
    // yield lookup to that component. Unlike `exact_rows`, this also counts the memory,
    // verify_instruction and const-table relations.
    pub lookup_rows: IndexMap<String, usize>,
    // The exact number of rows this component adds to each lookup relation it depends on, directly
    // or transitively. Same as `rows_upper_bound` but without the power-of-two padding, so it has
    // the same keys and its values are at most the corresponding upper bounds. Like
    // `rows_upper_bound`, it excludes the memory, verify_instruction and const-table relations.
    // Note that since intermediate component tables are padded to a power of two in the prover,
    // the real number of rows is at least this value; i.e. this is a lower bound on the actual
    // row counts.
    pub exact_rows: IndexMap<String, usize>,
    pub padding_type: PaddingType,
    // The number of cells in each row, including interaction columns.
    pub total_num_trace_cols: usize,
    // To this we should add the number of trace cells in:
    // - Const tables and their corresponding lookup components (multiplicity and logup columns)
    // - The memory tables (and their corresponding multiplicity and logup columns)
    // - The table of verify instruction (with number of rows equals the number of different pc
    //   values)
    pub trace_cells_upper_bound: usize,
    // An upper bound on the multiplicity values for lookups to const tables and memory tables.
    pub uses_upper_bound: IndexMap<String, usize>,
    // An upper bound on the number of rows in the trace for each called lookup relation.
    pub rows_upper_bound: IndexMap<String, usize>,
    // The uses upper bound is limited by the size of the field.
    pub max_instances_uses_limit: usize,
    // The rows upper bound is currently limited to 2**27.
    pub max_instances_rows_limit: usize,
}

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
            output.as_felts().into_iter().flat_map(|f| f.get_used_constraint_intermediates()),
        );
        let input_name = AirFnEntry::input_name(&air_fn.name());
        let input_limbs_mask = (0..joined_input.as_felts().len())
            .map(|i| {
                let name =
                    joined_input.get_limb_name(&input_name, i, &air_fn.input_expr_descriptions());
                constraint_intermediates.contains(&name)
            })
            .collect();
        let output_limbs_mask =
            output.as_felts().into_iter().map(|f| !f.is_directly_in_state()).collect();

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
    pub fn compile(&self, called_fns: &Ref<'_, IndexMap<String, Rc<AirFnEntry>>>) -> CompiledAirFn {
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
                        ab.get_public_params().iter().map(|p| p.name()).collect(),
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
                (relation_name.clone(), (component_name.clone(), relation_index, *max_inputs))
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
                self.filter_output_limbs(self.output.clone()).compile(CompileFor::Constraints),
                self.output_limb_names(Self::output_name(&self.name)),
            ),
            state_names: self.state.get_state_names(),
            constraint_lookups: self.air_body.get_constraint_lookups(),
            sub_components,
            n_inputs_added_per_relation,
            inline_calls,
            constraints: self.air_body.compile_for_constraints(),
            deductions: self.air_body.compile_for_deductions(),
            public_params: self.air_body.get_public_params().iter().map(|p| p.name()).collect(),
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
        format!("{air_fn_name}_{OUTPUT_VAR_SUFFIX}")
    }

    pub fn input_name(air_fn_name: &String) -> String {
        format!("{air_fn_name}_{INPUT_VAR_SUFFIX}")
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
        assert_eq!(input.as_felts().len(), self.input_limbs_mask.len());
        AirVarImpl::Array(
            self.input_limbs_mask
                .iter()
                .zip(input.as_felts())
                .filter_map(|(b, f)| b.then(|| f.into()))
                .collect(),
        )
    }

    pub fn input_limbs_type(&self) -> String {
        self.filter_input_limbs(self.joined_input.clone()).prover_type()
    }

    // Given the name of the output intermediate variable, uses the <output_expr_descriptions> and
    // returns the names of the limbs of the output (the felts that are in the mask).
    pub fn output_limb_names(&self, name: String) -> Vec<String> {
        self.output_limbs_mask
            .iter()
            .enumerate()
            .filter(|&(_, &b)| b)
            .map(|(i, _)| self.output.get_limb_name(&name, i, &self.output_expr_descriptions))
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
            .filter(|&(_, &b)| b)
            .map(|(i, _)| self.joined_input.get_limb_name(&name, i, &self.input_expr_descriptions))
            .collect::<Vec<_>>()
    }
}

// AirFnRegistry is created for a specific air function. It keeps all the air function entries
// for the air function and its subroutines.
#[derive(Debug, Clone, Default)]
pub struct AirFnRegistry {
    pub air_fns: Rc<RefCell<IndexMap<String, Rc<AirFnEntry>>>>,
    pub air_fn_ids: Rc<RefCell<HashSet<String>>>,
    pub relation_ids: Rc<RefCell<HashMap<String, M31>>>,
    pub public_params: PublicParams,
}

impl AirFnRegistry {
    pub fn new_empty() -> Self {
        Self {
            air_fns: Rc::new(RefCell::new(IndexMap::new())),
            air_fn_ids: Rc::new(RefCell::new(HashSet::new())),
            relation_ids: Rc::new(RefCell::new(HashMap::new())),
            public_params: Default::default(),
        }
    }

    pub fn new<E, I, O>(air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>) -> (Self, Rc<AirFnEntry>)
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
    ) -> Rc<AirFnEntry>
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

        for relation_name in air_fn.relation_names() {
            if !self.relation_ids.borrow().contains_key(&relation_name) {
                let id = random_m31(&relation_name);
                assert!(
                    !self.relation_ids.borrow().values().any(|x| *x == id),
                    "Relation with the same ID already exists"
                );

                self.relation_ids.borrow_mut().insert(relation_name.to_string(), id);
            }
        }

        let (air_body, state, ext_input, input, output) = self.build_air(air_fn, air_fn_id);
        let entry = AirFnEntry::new(air_fn, air_body, state, ext_input, input, output);

        // Don't test inline AirFns. It is OK to have an inline AirFn with a disconnected
        // constraint graph. For example, an AirFn that receives a Felt252 as 28 M31s and checks
        // that each of the M31s contains 9 bits.
        if entry.trace_type != TraceType::Inline {
            assert_constraint_graph_connected(&entry);
        }

        let entry_rc = Rc::new(entry);
        self.air_fns.borrow_mut().insert(air_fn.name(), entry_rc.clone());

        entry_rc
    }

    // Runs the air function on a given input and returns the resulting state and output.
    #[cfg(any(test, feature = "test"))]
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
    #[cfg(any(test, feature = "test"))]
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
            | TraceType::Opcode
            | TraceType::Relation => air_fn.lookup_call(&mut air_builder, ext_input, input),
            // For constant AirFns there are no constraints or deductions, so we just return the
            // output.
            TraceType::Const => {
                let output = air_fn.call(&mut air_builder, ext_input, input);
                assert!(output.clone().into().is_const(), "Output must be a constant");
                output
            }
            TraceType::Gate => air_fn.gate_call(&mut air_builder, ext_input, input),
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
        let input = I::new(input_name.clone(), Some(1));
        let input = input.rec_let(input_name, air_fn.input_expr_descriptions()).0;

        let mut air_builder = AirBuilder {
            component_context: Default::default(),
            air_body: AirBody::default(),

            // The row number doesn't influence the generated air_body.
            #[cfg(any(test, feature = "test"))]
            row_number: None,
            #[cfg(any(test, feature = "test"))]
            run: false,
            registry: self.clone(),
            intermediate_id: Rc::new(RefCell::new((air_fn_id, 0))),
        };

        if air_fn.trace_type() == TraceType::Inline {
            air_builder.component_context.enabler = if air_builder.is_run_mode() {
                const_expr!(1)
            } else {
                let mut enabler = FeltExpr::new("enabler".to_string(), Some(1));
                enabler.set_value(ValueInfo::Enabler);
                enabler
            };
        }
        let output = match air_fn.trace_type() {
            // For constant AirFns the value of <output> is meaningless, as we don't
            // output any constraints or deductions. It just has to be of the correct type.
            TraceType::Inline | TraceType::Builtin | TraceType::Const => {
                air_fn.call(&mut air_builder, ext_input.clone(), input.clone())
            }
            TraceType::Component
            | TraceType::ChainRound
            | TraceType::Opcode
            | TraceType::Memory
            | TraceType::Relation => {
                let output = air_fn.lookup_call(&mut air_builder, ext_input.clone(), input.clone());
                // Make sure that all intermediate variables in the output are visible in the
                // trace generation code, since this code returns the output.
                assert!(
                    output.clone().into().visibility().in_deductions,
                    "Output must have no intermediate variables that are not in deductions",
                );
                output
            }
            TraceType::Gate => {
                let output = air_fn.gate_call(&mut air_builder, ext_input.clone(), input.clone());
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
            .map(|(name, entry)| (name.clone(), entry.compile(&self.air_fns.borrow())))
            .collect()
    }

    pub fn collect_stats(&self) -> IndexMap<String, AirFnStat> {
        let mut stat = IndexMap::new();

        for (_, entry) in self.air_fns.borrow().iter() {
            if entry.trace_type == TraceType::Const || entry.trace_type == TraceType::Inline {
                continue;
            }

            self.add_entry_statistics(entry, &mut stat);
        }

        stat
    }

    // Collects statistics on a component.
    // trace_cells_upper_bound: An upper bound on the number of cells added to the trace for each
    // `AddInput` to this component. Includes rows added to other lookup components called by
    // this component. Doesn't include cells from components that are always filled
    // by the prover, regardless of whether they're called or not (e.g. const tables, the memory,
    // verify instruction).
    // We compute rows and uses upper bounds for a (possibly indirect) callee C by assuming that all
    // intermediate components are padded to a power of two, but ignoring the padding of C itself.
    // For example, if each row in component A calls component B 3 times, and each row in B calls C
    // 5 times, A.rows_upper_bound.get("C") will be 20, because we'll pad 3 to 4, but won't pad 5.
    // For uses_upper_bound, no need to pad the last component. For rows_upper_bound, the padding of
    // the last component is accounted for later, when computing `max_instances_rows_limit`.
    // We do that instead of using the padded count for all calls from the start because it gives
    // tighter upper bounds.
    fn add_entry_statistics(&self, entry: &AirFnEntry, stat: &mut IndexMap<String, AirFnStat>) {
        // Calculate number of trace columns.
        let num_state_cols = entry.state.get_state_names().len();
        let constraint_lookups = entry.air_body.get_constraint_lookups();
        let num_lookup_cols: usize = constraint_lookups.len();

        let total_num_trace_cols = num_state_cols
            + num_lookup_cols.div_ceil(LOOKUPS_PER_BATCH) * TRACE_COLUMNS_PER_LOOKUP_BATCH;

        // Calculate use and yield lookups.
        let mut use_lookup_cols: IndexMap<String, usize> = IndexMap::new();
        for (name, _) in
            constraint_lookups.iter().filter(|(_, use_or_yield)| *use_or_yield == UseOrYield::Use)
        {
            *use_lookup_cols.entry(name.clone()).or_default() += 1;
        }
        let mut yield_lookup_cols: IndexMap<String, usize> = IndexMap::new();
        for (name, _) in
            constraint_lookups.iter().filter(|(_, use_or_yield)| *use_or_yield == UseOrYield::Yield)
        {
            *yield_lookup_cols.entry(name.clone()).or_default() += 1;
        }

        let mut trace_cells_upper_bound = total_num_trace_cols;
        let mut uses_upper_bound = IndexMap::new();
        let mut rows_upper_bound = IndexMap::new();

        let reg = self.air_fns.borrow();
        let lookup_rows = entry.air_body.get_n_inputs_added_per_relation();

        // The exact number of rows this component adds to each lookup relation it depends on,
        // directly or transitively. Computed below like `rows_upper_bound`, but using the raw call
        // count `cnt` instead of the power-of-two-rounded `rounded_cnt`.
        let mut exact_rows: IndexMap<String, usize> = IndexMap::new();

        // The registry is ordered such that each component appears after its callees, so all
        // callees already have their statistics computed.
        for (relation_name, (air_fn_name, cnt)) in lookup_rows.iter() {
            // TODO(AnatG): Consider using relation name without converting to snake case.
            let name = relation_name.to_case(Case::Snake);

            // We round the number of times a lookup is called up to the next power of two, since
            // the statistics apply also to the padded rows.
            let rounded_cnt = if *cnt == 0 { 0 } else { cnt.next_power_of_two() };

            let called_entry = reg.get(air_fn_name).expect("Called entry not found in registry");
            let compiled_called_entry = called_entry.compile(&reg);
            let entry_stats = stat.get(air_fn_name).expect("Called entry not found in registry");

            if (compiled_called_entry.r#type != TraceType::Memory
                && compiled_called_entry.name != "verify_instruction")
                && called_entry.ext_input.is_none()
            {
                // Update trace cells upper bound for the current component.
                trace_cells_upper_bound += rounded_cnt * entry_stats.trace_cells_upper_bound;

                // Record the exact number of rows added to `name` directly (`cnt`) plus the rows
                // `name` adds transitively to its own dependencies (already exact in
                // `entry_stats.exact_rows`). Unlike `rows_upper_bound`, the transitive term uses
                // the raw `cnt` rather than `rounded_cnt`.
                *exact_rows.entry(name.clone()).or_default() += cnt;
                entry_stats.exact_rows.iter().for_each(|(row_name, row_bound)| {
                    *exact_rows.entry(row_name.clone()).or_default() += cnt * row_bound;
                });

                // Update rows upper bound for all lookup components.
                // The number of rows of each component is padded at the end, after adding all
                // inputs from all calls to it.
                *rows_upper_bound.entry(name.clone()).or_default() += cnt;
                entry_stats.rows_upper_bound.iter().for_each(|(row_name, row_bound)| {
                    *rows_upper_bound.entry(row_name.clone()).or_default() +=
                        rounded_cnt * row_bound;
                });
            }

            // Update uses upper bound for the lowest level lookup components (that have no
            // callees).
            if entry_stats.uses_upper_bound.is_empty() {
                // The called AirFn is a leaf AirFn (doesn't use other AirFns)
                assert!(
                    compiled_called_entry.padding_type == PaddingType::Multiplicity,
                    "If the entry doesn't use any other components, it must be padded with \
                     multiplicity."
                );
                *uses_upper_bound.entry(name.clone()).or_default() += cnt;
            } else {
                // The called AirFn is not a leaf AirFn
                entry_stats.uses_upper_bound.iter().for_each(|(use_name, use_bound)| {
                    *uses_upper_bound.entry(use_name.clone()).or_default() +=
                        rounded_cnt * *use_bound;
                });
            }
        }

        let max_instances_uses_limit =
            PRIME as usize / *uses_upper_bound.values().max().unwrap_or(&1);
        let max_instances_rows_limit =
            MAX_ROWS_PER_TABLE / rows_upper_bound.values().max().unwrap_or(&1).next_power_of_two();

        stat.insert(
            entry.name.clone(),
            AirFnStat {
                trace_type: entry.trace_type,
                log_height: entry.log_height,
                num_state_cols,
                use_lookup_cols,
                yield_lookup_cols,
                lookup_rows: lookup_rows.into_iter().map(|(r, (_, c))| (r, c)).collect(),
                exact_rows,
                padding_type: entry.padding_type,
                total_num_trace_cols,
                trace_cells_upper_bound,
                uses_upper_bound,
                rows_upper_bound,
                max_instances_uses_limit,
                max_instances_rows_limit,
            },
        );
    }
}
