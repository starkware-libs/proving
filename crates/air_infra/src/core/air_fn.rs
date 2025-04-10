use std::any::type_name;
use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use compiled_casm_air::compiled_structs::{TraceType, UseOrYield};
use compiled_casm_air::public_params::PublicParam;
use compiled_casm_air::relations::OPCODES_RELATION_NAME;
use compiled_casm_air::utils::{
    INTERMEDIATE_VAR_SUFFIX, OUTPUT_VAR_SUFFIX, STATE_INPUT_VAR, STATE_OUTPUT_VAR_SUFFIX,
};
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
use crate::airs::casm::const_tables::seq::*;
use crate::const_expr;

pub const MAX_NAME_LEN: usize = 50;

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
        let mut name = type_name::<Self>().to_string();
        name = name
            .find('<')
            .map(|i| name[..i].to_string())
            .unwrap_or(name);
        name = name
            .rfind("::")
            .map(|i| name[i + 2..].to_string())
            .unwrap_or(name);

        let mut res = format!(
            "{}_{}",
            name,
            serde_json::to_string_pretty(&self.inst_def()).expect("Failed to serialize inst def")
        );
        res = res
            .chars()
            .map(|x| match x {
                ' ' | ':' | '{' | '}' | '\n' | ',' | '[' | ']' | '<' | '>' => '_',
                _ => x,
            })
            .collect();
        res = res.replace('\"', "");
        res = res.replace("_true", "");
        while res.contains("__") {
            res = res.replace("__", "_");
        }
        if res.ends_with('_') {
            res.pop();
        }
        let pattern = Regex::new(r"_-([0-9]+)").unwrap();
        res = pattern.replace_all(&res, "_tmpminus_$1").to_string();
        res = res.to_case(Case::Snake);
        res = res.replace("_tmpminus_", "_m");
        if res.len() < MAX_NAME_LEN {
            res
        } else {
            format!("{}_{}", name.to_case(Case::Snake), self.hash())
        }
    }

    fn relation_name(&self) -> Option<String> {
        match self.trace_type() {
            TraceType::Component | TraceType::ChainRound => Some(self.name().to_case(Case::Pascal)),
            TraceType::Const => None,
            TraceType::Builtin => None,
            TraceType::Opcode => Some(OPCODES_RELATION_NAME.to_string()),
            TraceType::Memory => Some(self.name().to_case(Case::Pascal)),
            TraceType::Inline => None,
        }
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

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        match self.trace_type() {
            TraceType::Opcode => Some(vec![
                Some("pc".to_string()),
                Some("ap".to_string()),
                Some("fp".to_string()),
            ]),
            _ => None,
        }
    }

    fn output_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        match self.trace_type() {
            TraceType::Opcode => Some(vec![
                Some("pc".to_string()),
                Some("ap".to_string()),
                Some("fp".to_string()),
            ]),
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
            &format!("{}_{}", self.name(), OUTPUT_VAR_SUFFIX),
            self.output_expr_descriptions(),
        )
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
                || self.trace_type() == TraceType::Memory,
            "AirFn must be a component, chain round, opcode or memory"
        );

        Self::ExtIn::to_state(&mut ext_input);

        // Handle input
        if self.trace_type() == TraceType::Memory {
            // Memory - Assume input & output are already in state (filled by Stwo)
            for felt in input.as_felts_mut() {
                air_builder
                    .component_context
                    .state_mut()
                    .add(felt, STATE_INPUT_VAR);
            }

            let mut output = Self::Out::new("".to_string(), None);
            for felt in output.as_felts_mut() {
                air_builder.component_context.state_mut().add(
                    felt,
                    &format!("{}_{}", self.name(), STATE_OUTPUT_VAR_SUFFIX),
                );
            }
        } else {
            // Anything else - deduce input
            #[cfg(test)]
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
            // Chain components - use the input and yield the output
            air_builder.air_body.push(AirBodyComponent::LookupTerm {
                relation_name: self.relation_name().expect("Relation name not set"),
                felts: ext_input
                    .as_felts()
                    .into_iter()
                    .chain(input.as_felts())
                    .collect(),
                use_or_yield: UseOrYield::Use,
            });

            air_builder.air_body.push(AirBodyComponent::LookupTerm {
                relation_name: self.relation_name().expect("Relation name not set"),
                felts: output.as_felts(),
                use_or_yield: UseOrYield::Yield,
            });
        } else {
            // Other components - just yield the output
            air_builder.air_body.push(AirBodyComponent::LookupTerm {
                relation_name: self.relation_name().expect("Relation name not set"),
                felts: ext_input
                    .as_felts()
                    .into_iter()
                    .chain(input.as_felts())
                    .chain(output.as_felts())
                    .collect(),
                use_or_yield: UseOrYield::Yield,
            });
        }

        output
    }

    fn deduce_output(&self) -> Option<String> {
        if (self.trace_type() != TraceType::ChainRound && self.trace_type() != TraceType::Component)
            || Self::Out::is_empty()
        {
            return None;
        }
        panic!("deduce_output not implemented for this AirFn");
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

// Seperated from the air fn trait to support automated implementation
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
    pub(super) component_context: ComponentContext,
    pub(super) air_body: AirBody,
    #[cfg(test)]
    pub(super) row_number: Option<usize>,
    #[cfg(test)]
    pub(super) run: bool,
    pub(super) registry: AirFnRegistry,
    // TODO(AnatG): Move this to component_context.
    pub(super) intermediate_id: Rc<RefCell<(String, usize)>>,
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

    // TODO(Anat): Remove once we have row_index.
    // Should be used only within a lookup component, prior to calling
    // a constant table (with call_external_column).
    #[cfg(test)]
    pub fn set_row_number(&mut self, row_number: Option<usize>) {
        self.row_number = row_number;
    }

    pub fn constrain(&mut self, expr: FeltExpr, desc: &str) {
        #[cfg(test)]
        if self.run {
            assert!(
                expr.calc() == 0.to_string(),
                "Added incorrect constraint (does not evalutate to 0)"
            )
        }

        self.air_body.push(AirBodyComponent::Constraint(
            expr,
            (!desc.is_empty()).then(|| desc.to_string()),
        ));
    }

    pub fn deduce(&mut self, expr: &mut FeltExpr, desc: &str) -> FeltExpr {
        #[cfg(test)]
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
        #[cfg(test)]
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

        if let AirVarImpl::Expr(ExprImpl::Felt(f)) = var.clone().into() {
            if f.is_directly_in_state() {
                return var.clone();
            }
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
        self.air_body.push(AirBodyComponent::Intermediate(
            Intermediate::new_for_constraint(&name, &expr),
        ));
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
            return (expr, "()".to_string());
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
        self.registry.add_entry(air_fn);

        let mut air_builder = Self {
            component_context: self.component_context.clone(),
            air_body: AirBody::default(),
            #[cfg(test)]
            row_number: self.row_number,
            #[cfg(test)]
            run: self.run,
            registry: self.registry.clone(),
            intermediate_id: self.intermediate_id.clone(),
        };
        let state_offset = self.component_context.state().get_state_names().len();
        let (output, output_name) = air_fn.call_wrapper(&mut air_builder, (), input.clone());
        let state_names_after = self.component_context.state().get_state_names();

        self.air_body.push(AirBodyComponent::Call(Call {
            air_fn_name: air_fn.name(),
            air_fn_description: air_fn.description(),
            input: input.into(),
            output_name,
            output_expr_descriptions: air_fn.output_expr_descriptions(),
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
        assert!(
            air_fn.trace_type() == TraceType::Component,
            "Cannot lookup call AirFn {} - it is not a component",
            air_fn.name()
        );

        // Make sure the callee is in the registry
        self.registry.add_entry(air_fn);

        let output_name = (!O::is_empty()).then(|| {
            self.get_intermediate_name(Some(format!("{}_{}", air_fn.name(), OUTPUT_VAR_SUFFIX)))
        });
        let mut output = self.lookup_add_input_and_compute(
            air_fn,
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

        self.air_body.push(AirBodyComponent::LookupTerm {
            relation_name: air_fn.relation_name().expect("Relation name not set"),
            felts: ext_input
                .as_felts()
                .into_iter()
                .chain(input.as_felts())
                .chain(output.as_felts())
                .collect(),
            use_or_yield: UseOrYield::Use,
        });

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

        assert!(
            !S::is_empty(),
            "The input to a chain lookup call must not be empty."
        );

        // Make sure the callee is in the registry
        self.registry.add_entry(air_fn);

        let mut output_name = "".to_string();
        let mut output = <(ChainIdVar, RoundNumVar, S)>::new("".to_string(), None);

        let call_index = self.component_context.get_chain_call_index(air_fn);
        let chain_id = self.call_external_table(&Seq {})
            * const_expr!(air_fn.number_of_chains() as u32)
            + const_expr!(call_index);
        let mut input = (chain_id, const_expr!(first_round as u32), state);

        // Yield the input to the first round.
        self.air_body.push(AirBodyComponent::LookupTerm {
            relation_name: air_fn.relation_name().expect("Relation name not set"),
            felts: input.as_felts(),
            use_or_yield: UseOrYield::Yield,
        });

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
                (),
                input.clone(),
                Some(output_name.clone()),
            );

            // Prepare the input for the next round.
            input = (
                input.0.clone(),
                input.1.clone() + const_expr!(1),
                output.2.clone(),
            );
        }

        // TODO(AnatG): Consider not deducing the const parts of the output.
        // Deduce the output of the last round.
        // Output already has name <output_name>, but in run mode it might not be a variable.
        output = output.let_for_deduction(output_name).0;
        self.deduce_intermediate_var(
            &mut output,
            &format!("{}_{}", air_fn.name(), STATE_OUTPUT_VAR_SUFFIX),
            air_fn.output_expr_descriptions(),
        );

        // Use the output of the last round.
        self.air_body.push(AirBodyComponent::LookupTerm {
            relation_name: air_fn.relation_name().expect("Relation name not set"),
            felts: output.as_felts(),
            use_or_yield: UseOrYield::Use,
        });

        output.2
    }

    fn lookup_add_input_and_compute<E, I, O>(
        &mut self,
        air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>,
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
            air_fn_name: air_fn.name(),
            ext_input: ext_input_option.clone(),
            input: input_option.clone(),
        });

        #[cfg(test)]
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
            self.air_body.push(AirBodyComponent::LookupCall(LookupCall {
                air_fn_name: air_fn.name(),
                method_name: air_fn
                    .deduce_output()
                    .expect("No deduce_output method name"),
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
    pub(super) fn mem_read_unverified<K, V>(&mut self, memory: &dyn IsMemory<K, V>, key: &K::T) -> V
    where
        K: ExtTable,
        V: AirVar,
    {
        // Make sure the memory is in the registry
        self.registry.add_entry(memory);

        let value_name = self.get_intermediate_name(Some(format!("{}_value", memory.name())));
        #[allow(unused_mut)]
        let mut value = V::new(value_name.clone(), None);

        self.air_body.push(AirBodyComponent::LookupCall(LookupCall {
            air_fn_name: memory.name(),
            method_name: format!(
                "{}::deduce_output",
                memory.relation_name().expect("Relation name not found")
            ),
            ext_input: Some(key.clone().into()),
            input: None,
            output_name: value_name.clone(),
            output: value.clone().into(),
        }));

        #[cfg(test)]
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

        #[cfg(test)]
        if self.run {
            assert_eq!(
                memory
                    .mem()
                    .get(key)
                    .expect("Key doesn't exist in memory")
                    .to_values(),
                value.to_values(),
                "given value != value in memory"
            );
        }

        self.air_body.push(AirBodyComponent::LookupAddInput {
            air_fn_name: memory.name(),
            ext_input: Some(key.clone().into()),
            input: None,
        });
        self.air_body.push(AirBodyComponent::LookupTerm {
            relation_name: memory.relation_name().expect("Relation name not set"),
            felts: key.as_felts().into_iter().chain(value.as_felts()).collect(),
            use_or_yield: UseOrYield::Use,
        });
    }

    #[allow(unused_variables)]
    pub fn call_external_table<O>(&mut self, ext_table: &O) -> O::T
    where
        O: ExtTable,
    {
        let air_fn = ExtTableAirFn {
            ext_table: ext_table.clone(),
        };

        // Make sure the callee is in the registry
        self.registry.add_entry(&air_fn);

        #[cfg(test)]
        if self.run {
            let mut air_builder = Self {
                component_context: Default::default(),
                air_body: AirBody::default(),
                #[cfg(test)]
                row_number: self.row_number,
                #[cfg(test)]
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
            Some(desc) => format!("{}_{}", desc, suffix),
            None => suffix,
        };

        // Increase the intermediate index.
        self.intermediate_id.borrow_mut().1 += 1;

        name
    }
}
