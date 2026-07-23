use std::any::type_name;
use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use air_common::utils::fix_str;
use air_common::{GATE_RELATION_NAME, OPCODES_RELATION_NAME, PaddingType, TraceType, UseOrYield};
use convert_case::{Case, Casing};
use regex::Regex;
use serde::Serialize;

use super::air_body::*;
use super::air_fn_registry::*;
use super::expressions::expr::*;
use super::expressions::felt_expr::*;
use super::memory::*;
use super::state::*;
use super::variables::*;
use crate::const_expr;
use crate::core::expressions::var_expr::*;
use crate::core::public_params::PublicParam;
use crate::seq::*;

pub const MAX_NAME_LEN: usize = 50;
pub const INTERMEDIATE_VAR_SUFFIX: &str = "tmp";
pub const STATE_VAR_SUFFIX: &str = "col";
pub const INPUT_VAR_SUFFIX: &str = "input";
pub const OUTPUT_VAR_SUFFIX: &str = "output";
pub const STATE_INPUT_VAR: &str = "input";
pub const STATE_OUTPUT_VAR_SUFFIX: &str = "output";

// An air function should define a struct that implements the AirFn trait.
// The AirFn trait has two associated types, In and Out, which are the input and output types of the
// air function. It also defines whether the input is in the trace or not.
// The call method is the main method of the air function, and is used to build and run the air
// function.
pub trait AirFn: Debug + InstDefTrait {
    type ExtIn: ExtTable;
    type In: AirVar;
    type Out: AirVar;

    fn name(&self) -> String {
        let name = self.get_type_name();
        let inst_def = self.get_inst_def();

        if inst_def.is_empty() {
            return name;
        }

        if name.len() + inst_def.len() < MAX_NAME_LEN {
            return format!("{name}_{inst_def}");
        }

        format!("{name}_{}", self.hash())
    }

    fn get_type_name(&self) -> String {
        let mut name = type_name::<Self>().to_string();
        name = name.find('<').map(|i| name[..i].to_string()).unwrap_or(name);
        name = name.rfind("::").map(|i| name[i + 2..].to_string()).unwrap_or(name);
        fix_str(name)
    }

    fn get_inst_def(&self) -> String {
        let mut inst =
            serde_json::to_string_pretty(&self.inst_def()).expect("Failed to serialize inst def");
        inst = inst
            .chars()
            .map(|x| match x {
                ' ' | ':' | '{' | '}' | '\n' | ',' | '[' | ']' | '<' | '>' => '_',
                _ => x,
            })
            .collect();
        inst = inst.replace('\"', "");
        inst = inst.replace("_true", "");
        let pattern = Regex::new(r"_-([0-9]+)").unwrap();
        inst = pattern.replace_all(&inst, "_tmpminus_$1").to_string();
        inst = fix_str(inst);
        inst.replace("_tmpminus_", "_m")
    }

    fn relation_names(&self) -> Vec<String> {
        match self.trace_type() {
            TraceType::Component | TraceType::ChainRound => vec![self.name().to_case(Case::Pascal)],
            TraceType::Opcode => vec![OPCODES_RELATION_NAME.to_string()],
            TraceType::Memory => vec![self.name().to_case(Case::Pascal)],
            TraceType::Const | TraceType::Builtin | TraceType::Inline => vec![],
            TraceType::Gate => vec![GATE_RELATION_NAME.to_string()],
            TraceType::Relation => vec![self.name().to_case(Case::Pascal)],
        }
    }

    fn relation_name(&self) -> Option<String> {
        assert!(
            self.relation_names().len() <= 1,
            "AirFn {} has multiple relation names",
            self.name()
        );
        self.relation_names().first().cloned()
    }

    fn description(&self) -> String {
        self.name().to_case(Case::Title)
    }

    fn hash(&self) -> String {
        let name = format!("{}{}", type_name::<Self>(), self.inst_def());
        let mut s = DefaultHasher::new();
        name.hash(&mut s);
        let res: u64 = s.finish();
        format!("{h:.*}", 5, h = format!("{:x}", res))
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Inline
    }

    fn padding_type(&self) -> PaddingType {
        match self.trace_type() {
            TraceType::Builtin | TraceType::Const | TraceType::Inline => PaddingType::None,
            TraceType::Memory => PaddingType::Multiplicity,
            TraceType::Component
                if self.name() == "verify_instruction" || self.name().contains("aggregator") =>
            {
                PaddingType::Multiplicity
            }
            TraceType::Component if !<<Self as AirFn>::ExtIn as ExtTable>::T::is_empty() => {
                PaddingType::Multiplicity
            }
            TraceType::Gate | TraceType::Relation => PaddingType::Multiplicity,
            _ => PaddingType::Enabler,
        }
    }

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        match self.trace_type() {
            TraceType::Opcode => {
                Some(vec![Some("pc".to_string()), Some("ap".to_string()), Some("fp".to_string())])
            }
            _ => None,
        }
    }

    fn output_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        match self.trace_type() {
            TraceType::Opcode | TraceType::ChainRound => self.input_expr_descriptions(),
            _ => None,
        }
    }

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        ext_input: <Self::ExtIn as ExtTable>::T,
        input: Self::In,
    ) -> Self::Out;

    fn call_wrapper(
        &self,
        air_builder: &mut AirBuilder,
        ext_input: <Self::ExtIn as ExtTable>::T,
        input: Self::In,
    ) -> (Self::Out, String) {
        let output = self.call(air_builder, ext_input, input);
        air_builder.let_with_name(
            output,
            &AirFnEntry::output_name(&self.name()),
            self.output_expr_descriptions(),
        )
    }

    fn gate_call(
        &self,
        air_builder: &mut AirBuilder,
        mut ext_input: <Self::ExtIn as ExtTable>::T,
        mut input: Self::In,
    ) -> Self::Out {
        assert!(self.trace_type() == TraceType::Gate, "AirFn must be a gate");

        Self::ExtIn::to_state(&mut ext_input);

        // Deduce input
        #[cfg(any(test, feature = "test"))]
        if air_builder.is_run_mode() {
            // In run mode the input might not be a variable - make it a variable.
            // The name is irrelevant in run mode.
            input = input.let_for_deduction("".to_string()).0;
        }
        air_builder.deduce_intermediate_var(
            &mut input,
            STATE_INPUT_VAR,
            self.input_expr_descriptions(),
        );

        // Perform AirFn logic
        self.call(air_builder, ext_input.clone(), input.clone())
    }

    fn lookup_call(
        &self,
        air_builder: &mut AirBuilder,
        mut ext_input: <Self::ExtIn as ExtTable>::T,
        mut input: Self::In,
    ) -> Self::Out {
        assert!(
            self.trace_type() == TraceType::Component
                || self.trace_type() == TraceType::ChainRound
                || self.trace_type() == TraceType::Opcode
                || self.trace_type() == TraceType::Memory
                || self.trace_type() == TraceType::Relation,
            "AirFn must be a component, chain round, opcode, memory or relation"
        );

        Self::ExtIn::to_state(&mut ext_input);

        // Add enabler / multiplicity columns
        let lookup_control_values = match self.padding_type() {
            PaddingType::Multiplicity => (0..self.relation_names().len())
                .map(|idx| {
                    let name = format!("multiplicity_{idx}");
                    let mut multiplicity = if air_builder.is_run_mode() {
                        const_expr!(1)
                    } else {
                        let mut multiplicity = FeltExpr::new(name.clone(), Some(1));
                        multiplicity.set_value(ValueInfo::Multiplicity(idx));
                        multiplicity
                    };
                    air_builder.deduce(&mut multiplicity, &format!("multiplicity_{idx}"));
                    multiplicity
                })
                .collect::<Vec<_>>(),
            PaddingType::Enabler => {
                let name = "enabler".to_string();
                let mut enabler = if air_builder.is_run_mode() {
                    const_expr!(1)
                } else {
                    let mut enabler = FeltExpr::new(name.clone(), Some(1));
                    enabler.set_value(ValueInfo::Enabler);
                    enabler
                };
                air_builder.deduce(&mut enabler, "enabler");

                air_builder.component_context.enabler = enabler.clone();

                air_builder.constrain(
                    enabler.clone() * enabler.clone() - enabler.clone(),
                    "Enabler is a bit",
                );
                vec![enabler]
            }
            PaddingType::None => panic!("Lookup call to an un-padded component"),
        };

        // Handle input
        if self.trace_type() == TraceType::Memory {
            // Memory - Assume input & output are already in state (filled by Stwo)
            for felt in input.as_felts_mut() {
                air_builder.component_context.state_mut().add(felt, STATE_INPUT_VAR);
            }

            let mut output = Self::Out::new("".to_string(), None);
            for felt in output.as_felts_mut() {
                air_builder
                    .component_context
                    .state_mut()
                    .add(felt, &format!("{}_{}", self.name(), STATE_OUTPUT_VAR_SUFFIX));
            }
        } else {
            // Anything else - deduce input
            #[cfg(any(test, feature = "test"))]
            if air_builder.is_run_mode() {
                // In run mode the input might not be a variable - make it a variable.
                // The name is irrelevant in run mode.
                input = input.let_for_deduction("".to_string()).0;
            }
            air_builder.deduce_intermediate_var(
                &mut input,
                STATE_INPUT_VAR,
                self.input_expr_descriptions(),
            );
        }

        // Perform AirFn logic
        let output = self.call(air_builder, ext_input.clone(), input.clone());

        // Add lookup terms
        if self.trace_type() == TraceType::Opcode || self.trace_type() == TraceType::ChainRound {
            // Chain components - use the input and yield the output gated by the enabler
            assert!(self.padding_type() == PaddingType::Enabler);
            air_builder.add_lookup_term(
                &self.relation_name().expect("Relation name not set"),
                ext_input.as_felts().into_iter().chain(input.as_felts()).collect(),
                UseOrYield::Use,
                lookup_control_values[0].clone(),
            );

            air_builder.add_lookup_term(
                &self.relation_name().expect("Relation name not set"),
                output.as_felts(),
                UseOrYield::Yield,
                lookup_control_values[0].clone(),
            );
        } else {
            // Other components - just yield the input and output
            if self.padding_type() == PaddingType::Enabler {
                // If we have a multiple-relation component with an enabler, the following
                // loop will have to be changed.
                assert_eq!(self.relation_names().len(), 1);
            }

            for (i, relation_name) in self.relation_names().iter().enumerate() {
                air_builder.add_lookup_term(
                    relation_name,
                    ext_input
                        .as_felts()
                        .into_iter()
                        .chain(input.as_felts())
                        .chain(output.as_felts())
                        .collect(),
                    UseOrYield::Yield,
                    lookup_control_values[i].clone(),
                );
            }
        }

        output
    }
}

pub trait ChainRoundAirFn<S>:
    AirFn<ExtIn = (), In = (ChainIdVar, RoundNumVar, S), Out = (ChainIdVar, RoundNumVar, S)>
where
    S: AirVar,
    (ChainIdVar, RoundNumVar, S): AirVar,
{
    // The number of calls to chain_lookup_call with this air_fn
    fn number_of_chains(&self) -> usize;
}

// Separated from the air fn trait to support automated implementation
pub trait InstDefTrait {
    fn inst_def(&self) -> serde_json::Value;
}

impl<F> InstDefTrait for F
where
    F: AirFn + Serialize,
{
    fn inst_def(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("Failed to serialize inst def")
    }
}

// AirBuilder is a struct that is used to build an air function.
// It is passed to the call method of an air function, and is used to add constraints, deductions,
// assignments and intermediate variables to the air function.
#[derive(Debug)]
pub struct AirBuilder {
    pub component_context: ComponentContext,
    pub air_body: AirBody,
    #[cfg(any(test, feature = "test"))]
    pub row_number: Option<usize>,
    #[cfg(any(test, feature = "test"))]
    pub run: bool,
    pub registry: AirFnRegistry,
    // TODO(AnatG): Move this to component_context.
    pub intermediate_id: Rc<RefCell<(String, usize)>>,
}

impl AirBuilder {
    #[cfg(any(test, feature = "test"))]
    pub fn is_run_mode(&self) -> bool {
        self.run
    }

    #[cfg(not(any(test, feature = "test")))]
    pub fn is_run_mode(&self) -> bool {
        false
    }

    #[cfg(any(test, feature = "test"))]
    pub fn row_number(&self) -> Option<usize> {
        self.row_number
    }

    // TODO(Anat): Remove once we have row_index.
    // Should be used only within a lookup component, prior to calling
    // a constant table (with call_external_column).
    #[cfg(any(test, feature = "test"))]
    pub fn set_row_number(&mut self, row_number: Option<usize>) {
        self.row_number = row_number;
    }

    pub fn add_lookup_term(
        &mut self,
        relation_name: &str,
        mut felts: Vec<FeltExpr>,
        use_or_yield: UseOrYield,
        multiplicity: FeltExpr,
    ) {
        let relation_id_expr = FeltExpr::Var(VarExpr::new_const(
            *self
                .registry
                .relation_ids
                .borrow()
                .get(relation_name)
                .unwrap_or_else(|| panic!("Relation {relation_name} not found")),
        ));
        felts.insert(0, relation_id_expr);
        self.air_body.push(AirBodyComponent::LookupTerm {
            relation_name: relation_name.to_owned(),
            felts,
            use_or_yield,
            multiplicity,
        });
    }

    pub fn constrain(&mut self, expr: FeltExpr, desc: &str) {
        #[cfg(any(test, feature = "test"))]
        if self.run {
            assert!(
                expr.calc() == 0.to_string(),
                "Added incorrect constraint (does not evaluate to 0)"
            )
        }

        self.air_body
            .push(AirBodyComponent::Constraint(expr, (!desc.is_empty()).then(|| desc.to_string())));
    }

    pub fn deduce(&mut self, expr: &mut FeltExpr, desc: &str) -> FeltExpr {
        #[cfg(any(test, feature = "test"))]
        if !self.run {
            // Cannot assert this in run mode, where we might deduce constants.
            assert!(!expr.is_const(), "Cannot deduce a constant");
        }

        self.air_body.push(AirBodyComponent::Deduction(
            expr.clone(),
            (!desc.is_empty()).then(|| desc.to_string()),
        ));
        self.component_context.state_mut().add(expr, desc);
        expr.clone()
    }

    pub fn assign(&mut self, expr: &mut FeltExpr, desc: &str) -> FeltExpr {
        #[cfg(any(test, feature = "test"))]
        if !self.run {
            // Cannot assert this in run mode, where we might deduce constants.
            assert!(!expr.is_const(), "Cannot assign a constant");
        }

        let before = expr.clone();
        self.component_context.state_mut().add(expr, desc);

        let constraint = expr.clone() - before.clone();
        self.air_body.push(AirBodyComponent::Assignment {
            constraint: constraint.clone(),
            deduction: before,
            desc: (!desc.is_empty()).then(|| desc.to_string()),
        });
        expr.clone()
    }

    pub fn deduce_air_var<V>(&mut self, mut var: V, desc: &str) -> V
    where
        V: AirVar,
    {
        if V::is_empty() {
            return var;
        }

        var = self.let_for_deduction(var, desc);
        self.deduce_intermediate_var(&mut var, desc, None);
        var
    }

    pub(super) fn deduce_intermediate_var<V>(
        &mut self,
        var: &mut V,
        desc: &str,
        expr_descriptions: Option<Vec<Option<String>>>,
    ) where
        V: AirVar,
    {
        let mut felts = var.as_felts_mut();
        if felts.len() == 1 {
            self.deduce(felts[0], desc);
            return;
        }

        // Make sure there are no state variables with names like "_pc" or "_limb_7_col202", where
        // it's not clear what the "limb" or "pc" refers to.
        assert!(
            !desc.is_empty(),
            "Intermediate variable description is required for deducing multiple felts"
        );

        let air_val_impl: AirVarImpl = var.clone().into();
        for (i, felt) in var.as_felts_mut().into_iter().enumerate() {
            let name = air_val_impl.get_limb_name(desc, i, &expr_descriptions);
            self.deduce(felt, &name);
        }
    }

    pub fn let_for_deduction<V>(&mut self, var: V, desc: &str) -> V
    where
        V: AirVar,
    {
        if V::is_empty() {
            return var;
        }

        if let AirVarImpl::Expr(ExprImpl::Felt(f)) = var.clone().into()
            && f.is_directly_in_state()
        {
            return var.clone();
        }

        let name = self.get_intermediate_name((!desc.is_empty()).then(|| desc.to_string()));
        let (res, interm) = var.let_for_deduction(name);
        self.air_body.push(AirBodyComponent::Intermediate(interm));
        res
    }

    pub fn let_for_constraint(&mut self, mut expr: FeltExpr, desc: &str) -> FeltExpr {
        if expr.is_directly_in_state() {
            return expr.clone();
        }

        let name = self.get_intermediate_name((!desc.is_empty()).then(|| desc.to_string()));
        self.air_body
            .push(AirBodyComponent::Intermediate(Intermediate::new_for_constraint(&name, &expr)));
        expr.let_for_constraint(name);
        expr
    }

    // For complex expressions, creates intermediate variables visible in constraints and deductions
    // for every felt of the expression, and an intermediate variable for the expression itself,
    // known only in deductions.
    // For a felt expression, creates a single intermediate variable visible in constraints and
    // deductions.
    pub fn let_<O>(&mut self, expr: O, desc: &str) -> O
    where
        O: AirVar,
    {
        self.let_with_name(expr, desc, None).0
    }

    pub(super) fn let_with_name<O>(
        &mut self,
        expr: O,
        desc: &str,
        expr_descriptions: Option<Vec<Option<String>>>,
    ) -> (O, String)
    where
        O: AirVar,
    {
        if O::is_empty() {
            return (expr, "[]".to_string());
        }

        let name = self.get_intermediate_name((!desc.is_empty()).then(|| desc.to_string()));
        let (new_expr, vars) = expr.rec_let(name.clone(), expr_descriptions);
        for var in vars {
            self.air_body.push(AirBodyComponent::Intermediate(var));
        }

        (new_expr, name)
    }

    pub fn call<I, O>(&mut self, air_fn: &dyn AirFn<ExtIn = (), In = I, Out = O>, input: I) -> O
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
            "Cannot call AirFn {} - it is not an inline AirFn",
            air_fn.name()
        );

        // Make sure the callee is in the registry
        let entry = self.registry.add_entry(air_fn);

        let mut air_builder = Self {
            component_context: self.component_context.clone(),
            air_body: AirBody::default(),
            #[cfg(any(test, feature = "test"))]
            row_number: self.row_number,
            #[cfg(any(test, feature = "test"))]
            run: self.run,
            registry: self.registry.clone(),
            intermediate_id: self.intermediate_id.clone(),
        };
        let state_offset = self.component_context.state().get_state_names().len();
        let (output, output_name) = air_fn.call_wrapper(&mut air_builder, (), input.clone());
        let state_names_after = self.component_context.state().get_state_names();

        self.air_body.push(AirBodyComponent::Call(Call {
            entry,
            enabler: self.component_context.enabler.clone(),
            input: input.into(),
            output_name,
            output: output.clone().into(),
            state_names: state_names_after[state_offset..].to_vec(),
            air_body: air_builder.air_body,
        }));

        output
    }

    pub fn lookup_call<E, I, O>(
        &mut self,
        air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>,
        ext_input: E::T,
        input: I,
    ) -> O
    where
        E: ExtTable,
        I: AirVar,
        O: AirVar,
    {
        self.lookup_call_variant(air_fn, ext_input, input, 0)
    }

    pub fn lookup_call_variant<E, I, O>(
        &mut self,
        air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>,
        ext_input: E::T,
        input: I,
        variant: usize,
    ) -> O
    where
        E: ExtTable,
        I: AirVar,
        O: AirVar,
    {
        assert!(
            air_fn.trace_type() == TraceType::Component
                || air_fn.trace_type() == TraceType::Relation,
            "Cannot lookup call AirFn {} - it is not a component/relation",
            air_fn.name()
        );

        // Make sure the callee is in the registry
        self.registry.add_entry(air_fn);

        let relation_name =
            air_fn.relation_names().get(variant).expect("Relation name not found").clone();

        let output_name = (!O::is_empty()).then(|| {
            self.get_intermediate_name(Some(AirFnEntry::output_name(
                &relation_name.to_case(Case::Snake),
            )))
        });
        let mut output = self.lookup_add_input_and_compute(
            air_fn,
            relation_name.clone(),
            ext_input.clone(),
            input.clone(),
            output_name.clone(),
        );

        // Deduce the output if it is not empty.
        if !O::is_empty() {
            // Output already has name <output_name>, but in run mode it might not be a variable.
            (output, _) = output.let_for_deduction(output_name.expect("Output name not set"));
            self.deduce_intermediate_var(
                &mut output,
                &format!("{}_{}", air_fn.name(), STATE_OUTPUT_VAR_SUFFIX),
                air_fn.output_expr_descriptions(),
            );
        }

        self.add_lookup_term(
            &relation_name,
            ext_input
                .as_felts()
                .into_iter()
                .chain(input.as_felts())
                .chain(output.as_felts())
                .collect(),
            UseOrYield::Use,
            self.component_context.enabler.clone(),
        );

        output
    }

    // Creates <num_of_rounds> rows in <air_fn> with consecutive round numbers.
    //
    // air_fn: a ChainRoundAirFn with ChainRound trace type.
    // input: The chain index (between 0 and NC - 1), first round number in this chain, and the
    // initial state for the first round.
    // num_of_rounds: The number of rows to create.
    //
    // Returns the output state of the last row.
    pub fn chain_lookup_call<S>(
        &mut self,
        air_fn: &dyn ChainRoundAirFn<S>,
        state: S,
        first_round: usize,
        num_of_rounds: usize,
    ) -> S
    where
        S: AirVar,
        (ChainIdVar, RoundNumVar, S): AirVar,
    {
        assert!(
            air_fn.trace_type() == TraceType::ChainRound,
            "Cannot chain call AirFn {} - it is not a chain round",
            air_fn.name()
        );

        assert!(!S::is_empty(), "The input to a chain lookup call must not be empty.");

        // Make sure the callee is in the registry
        self.registry.add_entry(air_fn);

        let mut output_name = "".to_string();
        let mut output = <(ChainIdVar, RoundNumVar, S)>::new("".to_string(), None);

        let chain_id = self.get_chain_id(air_fn);
        let mut input = (chain_id.clone(), const_expr!(first_round), state);

        // Yield the input to the first round.
        self.add_lookup_term(
            &air_fn.relation_name().expect("Relation name not set"),
            input.as_felts(),
            UseOrYield::Yield,
            self.component_context.enabler.clone(),
        );

        // TODO(AnatG): Consider adding all inputs to the lookup component together in one
        // LookupAddInput.
        for _ in 0..num_of_rounds {
            output_name = self.get_intermediate_name(Some(format!(
                "{}_output_round_{}",
                air_fn.name(),
                input.1.value().expect("The round number is always known")
            )));
            output = self.lookup_add_input_and_compute(
                air_fn,
                air_fn.relation_name().expect("Relation name not set"),
                (),
                input.clone(),
                Some(output_name.clone()),
            );

            // Prepare the input for the next round.
            input = (input.0.clone(), input.1.clone() + const_expr!(1), output.2.clone());
        }

        // Output already has name <output_name>, but in run mode it might not be a variable.
        output = output.let_for_deduction(output_name).0;

        // Deduce the output of the last round.
        let mut final_state = output.2;
        let final_state_descriptions =
            air_fn.output_expr_descriptions().map(|descrs| descrs.split_at(2).1.to_vec());
        self.deduce_intermediate_var(
            &mut final_state,
            &format!("{}_{}", air_fn.name(), STATE_OUTPUT_VAR_SUFFIX),
            final_state_descriptions,
        );

        // Use the output of the last round.
        self.add_lookup_term(
            &air_fn.relation_name().expect("Relation name not set"),
            (chain_id, const_expr!(first_round + num_of_rounds), final_state.clone()).as_felts(),
            UseOrYield::Use,
            self.component_context.enabler.clone(),
        );

        final_state
    }

    fn get_chain_id<S>(&mut self, air_fn: &dyn ChainRoundAirFn<S>) -> FeltExpr
    where
        S: AirVar,
        (ChainIdVar, RoundNumVar, S): AirVar,
    {
        #[cfg(any(test, feature = "test"))]
        if self.run {
            // The chain id is not relevant in run mode.
            return FeltExpr::default();
        }

        let call_index = self.component_context.get_chain_call_index(air_fn);
        let chain_tmp =
            self.component_context.get_chain_call_intermediate(air_fn).unwrap_or_else(|| {
                let mut chain_tmp =
                    self.call_external_table(&Seq {}) * const_expr!(air_fn.number_of_chains());
                chain_tmp = self.let_(chain_tmp, &format!("{}_chain_tmp", air_fn.name()));
                self.component_context.set_chain_call_intermediate(air_fn, chain_tmp.clone());
                chain_tmp
            });

        if call_index == 0 {
            chain_tmp
        } else {
            self.let_(chain_tmp + const_expr!(call_index), &format!("{}_chain_id", air_fn.name()))
        }
    }

    fn lookup_add_input_and_compute<E, I, O>(
        &mut self,
        air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>,
        relation_name: String,
        ext_input: E::T,
        input: I,
        output_name: Option<String>,
    ) -> O
    where
        E: ExtTable,
        I: AirVar,
        O: AirVar,
    {
        #[allow(unused_mut)]
        let mut output = O::new(output_name.clone().unwrap_or_default(), None);
        let ext_input_option = (!E::T::is_empty()).then(|| ext_input.clone().into());
        let input_option = (!I::is_empty()).then(|| input.clone().into());

        self.air_body.push(AirBodyComponent::LookupAddInput {
            relation_name: relation_name.clone(),
            air_fn_name: air_fn.name(),
            ext_input: ext_input_option.clone(),
            input: input_option.clone(),
        });

        #[cfg(any(test, feature = "test"))]
        if self.run {
            let mut air_builder = Self {
                component_context: Default::default(),
                air_body: AirBody::default(),
                // When we call a separate component using lookup, we access an arbitrary row in
                // that component (depending on how its rows are sorted). That is, the row number
                // in the callee is not related to the row number in the caller.
                row_number: None,
                run: self.run,
                registry: self.registry.clone(),
                // The intermediate_id is not used in run mode.
                intermediate_id: self.intermediate_id.clone(),
            };
            output = air_fn.lookup_call(&mut air_builder, ext_input.clone(), input.clone());
        }

        if !O::is_empty() {
            assert!(
                relation_name
                    == air_fn
                        .relation_name()
                        .expect("Called air_fn with output should have a single relation")
            );
            self.air_body.push(AirBodyComponent::LookupCall(LookupCall {
                air_fn_name: air_fn.name(),
                method_name: format!("{relation_name}::deduce_output"),
                ext_input: ext_input_option,
                input: input_option,
                output_name: output_name.expect("Output name not set"),
                output: output.clone().into(),
            }));
        }

        output
    }

    // Reads the value from the memory, creates an intermediate variable for the value, and returns
    // it. Does not add any constraints or deductions.
    pub(crate) fn mem_read_unverified<K, V>(&mut self, memory: &dyn IsMemory<K, V>, key: &K::T) -> V
    where
        K: ExtTable,
        V: AirVar,
    {
        // Make sure the memory is in the registry
        self.registry.add_entry(memory);

        let value_name = self.get_intermediate_name(Some(format!("{}_value", memory.name())));
        #[allow(unused_mut)]
        let mut value = V::new(value_name.clone(), None);

        let relation_name = memory.relation_name().expect("Relation name not found");
        self.air_body.push(AirBodyComponent::LookupCall(LookupCall {
            air_fn_name: memory.name(),
            method_name: format!("{relation_name}::deduce_output",),
            ext_input: Some(key.clone().into()),
            input: None,
            output_name: value_name.clone(),
            output: value.clone().into(),
        }));

        #[cfg(any(test, feature = "test"))]
        if self.run {
            let mut air_builder = Self {
                component_context: Default::default(),
                air_body: AirBody::default(),
                // This is None for the same reason as in lookup_call.
                row_number: None,
                run: self.run,
                registry: self.registry.clone(),
                // The intermediate_id is not used in run mode.
                intermediate_id: self.intermediate_id.clone(),
            };
            value = memory.lookup_call(&mut air_builder, key.clone(), ());
        }

        value.let_for_deduction(value_name).0
    }

    // Assumes the key and value are in the state (of the caller). Adds a lookup constraint
    // to verify that memory[key] == value.
    pub fn mem_verify<K, V>(&mut self, memory: &dyn IsMemory<K, V>, key: &K::T, value: V)
    where
        K: ExtTable,
        V: AirVar,
    {
        // Make sure the memory is in the registry
        self.registry.add_entry(memory);

        #[cfg(any(test, feature = "test"))]
        if self.run {
            assert_eq!(
                memory.mem().get(key).expect("Key doesn't exist in memory").to_values(),
                value.to_values(),
                "given value != value in memory"
            );
        }

        self.air_body.push(AirBodyComponent::LookupAddInput {
            relation_name: memory.relation_name().expect("Relation name not found"),
            air_fn_name: memory.name(),
            ext_input: Some(key.clone().into()),
            input: None,
        });
        self.add_lookup_term(
            &memory.relation_name().expect("Relation name not set"),
            key.as_felts().into_iter().chain(value.as_felts()).collect(),
            UseOrYield::Use,
            self.component_context.enabler.clone(),
        );
    }

    #[allow(unused_variables)]
    pub fn call_external_table<O>(&mut self, ext_table: &O) -> O::T
    where
        O: ExtTable,
    {
        let air_fn = ExtTableAirFn { ext_table: ext_table.clone() };

        // Make sure the callee is in the registry
        self.registry.add_entry(&air_fn);

        #[cfg(any(test, feature = "test"))]
        if self.run {
            let mut air_builder = Self {
                component_context: Default::default(),
                air_body: AirBody::default(),
                #[cfg(any(test, feature = "test"))]
                row_number: self.row_number,
                #[cfg(any(test, feature = "test"))]
                run: self.run,
                registry: self.registry.clone(),
                // The intermediate_id is not used in run mode.
                intermediate_id: self.intermediate_id.clone(),
            };
            return air_fn.call(&mut air_builder, (), ());
        }

        O::new()
    }

    pub fn get_public_param(&self, which: PublicParam) -> FeltExpr {
        self.registry.public_params.get(which)
    }

    fn get_intermediate_name(&mut self, desc: Option<String>) -> String {
        let suffix = format!(
            "{}_{}_{}",
            INTERMEDIATE_VAR_SUFFIX,
            self.intermediate_id.borrow().0,
            self.intermediate_id.borrow().1
        );
        let name = match desc {
            Some(desc) => format!("{desc}_{suffix}"),
            None => suffix,
        };

        // Increase the intermediate index.
        self.intermediate_id.borrow_mut().1 += 1;

        name
    }
}
