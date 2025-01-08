use std::any::type_name;
use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use compiled_casm_air::compiled_structs::UseOrYield;
use compiled_casm_air::public_params::PublicParam;
use compiled_casm_air::relations::OPCODES_RELATION_NAME;
use compiled_casm_air::utils::INTERMEDIATE_VAR_SUFFIX;
use convert_case::{Case, Casing};
use indexmap::IndexMap;
use prover_types::cpu::ProverType;
use serde::{Deserialize, Serialize};

use super::air_body::*;
use super::air_fn_registry::*;
use super::expressions::felt_expr::*;
use super::memory::*;
use super::state::*;
use super::variables::*;
use crate::airs::casm::const_tables::seq::*;
use crate::const_expr;
use crate::core::Felt;

pub const MAX_NAME_LEN: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceType {
    // Doesn't have its own component in the trace, always inlined into its caller.
    // Can be called only with call.
    Inline,

    // Has its own component in the trace. Each call generates a new row in that component.
    // Can be called only with lookup_call. Yields lookup data.
    Component,

    // Has its own component in the trace. The trace for this component is pre-filled with rows
    // for all possible inputs by external means. Doesn't generate deductions or constraints.
    // Has no input, only output. Can be called only with call_external_column. Doesn't yield
    // lookup data.
    Const,

    // Has its own component in the trace. Has no input and no output. Cannot be called from
    // another component. Doesn't yield lookup data.
    Builtin,

    // Has its own component in the trace. Its input and output are casm states.
    // Cannot be called from another component. Doesn't yield multiplicity column.
    // Generates accumulated sum column where the input
    // is used and the output is yielded (chain lookup constraint).
    // Their chain lookup relation is called OPCODES_RELATION_NAME.
    Opcode,

    // Memory components are pre-filled. Their trace consists of only input and output columns, or
    // only output columns, if the input is const. They don't generate deductions. They can
    // generate constraints, and they yield lookup data. They implement the IsMemory trait.
    Memory,

    // Has its own component in the trace. Its input and output are of the same type ([FeltExpr;
    // 2], S), where S is some AirVar. Doesn't yield multiplicity column.
    // Generates accumulated sum column where the input
    // is used and the output is yielded (chain lookup constraint).
    // Can be called only with chain_lookup_call.
    // Important: A ChainRound can be called from a single caller, and that caller can only call
    // it once in each row. This is because we use the caller Seq column to identify the chain
    // (see chain_lookup_call).
    ChainRound,
}

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
            res.to_case(Case::Snake)
        } else {
            format!("{}_{:x}", name.to_case(Case::Snake), self.hash())
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

    fn hash(&self) -> u64 {
        let name = format!("{}{:?}", type_name::<Self>(), InstDefTrait::inst_def(self));
        let mut s = DefaultHasher::new();
        name.hash(&mut s);
        s.finish()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Inline
    }

    // Returns whether the input of the air function should be in the trace when it is called.
    // If None, it means that the input is a tuple or array, and parts of it should be in the trace
    // and parts should not. In this case, the infra will not check if the input is in the trace.
    // (see CondDecodeSmallSign)
    fn input_in_trace(&self) -> Option<bool> {
        Some(self.trace_type() == TraceType::Const || self.trace_type() == TraceType::Inline)
    }

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        ext_input: <Self::ExtIn as ExtTable>::T,
        input: Self::In,
    ) -> Self::Out;

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
                air_builder.state.add(felt, "input");
            }

            let mut output = Self::Out::new("".to_string(), false);
            for felt in output.as_felts_mut() {
                air_builder
                    .state
                    .add(felt, &format!("{}_output", self.name()));
            }
        } else if !Self::In::is_empty() {
            // Anything else - deduce input
            input = air_builder.let_for_deduction(input, "input");
            if let Some(descs) = input.get_felt_descriptions() {
                for (felt, desc) in input.as_felts_mut().into_iter().zip(descs) {
                    air_builder.deduce(felt, &format!("input_{}", desc));
                }
            } else {
                for felt in input.as_felts_mut() {
                    air_builder.deduce(felt, "input");
                }
            }
        }

        // Perform AirFn logic
        let output = self.call(air_builder, ext_input.clone(), input.clone());

        // Add lookup terms
        if self.trace_type() == TraceType::Opcode || self.trace_type() == TraceType::ChainRound {
            // Chain components - use the input and yield the output
            air_builder.air_body.0.push(AirBodyComponent::LookupTerm {
                relation_name: self.relation_name().expect("Relation name not set"),
                felts: ext_input
                    .as_felts()
                    .into_iter()
                    .chain(input.as_felts())
                    .collect(),
                use_or_yield: UseOrYield::Use,
            });

            air_builder.air_body.0.push(AirBodyComponent::LookupTerm {
                relation_name: self.relation_name().expect("Relation name not set"),
                felts: output.as_felts(),
                use_or_yield: UseOrYield::Yield,
            });
        } else {
            // Other components - just yield the output
            air_builder.air_body.0.push(AirBodyComponent::LookupTerm {
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
}

// Seperated from the air fn trait to support automated implementation
pub trait InstDefTrait {
    fn inst_def(&self) -> IndexMap<String, String>;
}

// AirBuilder is a struct that is used to build an air function.
// It is passed to the call method of an air function, and is used to add constraints, deductions,
// assignments and intermediate variables to the air function.
#[derive(Debug)]
pub struct AirBuilder {
    pub(super) state: State,
    pub(super) air_body: AirBody,
    #[cfg(test)]
    pub(super) row_number: Option<usize>,
    #[cfg(test)]
    pub(super) run: bool,
    pub(super) registry: AirFnRegistry,
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

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn constrain(&mut self, expr: FeltExpr, desc: &str) {
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

        assert!(
            expr.visibility().in_constraints,
            "Constraint contains an intermediate variable that is not in constraints"
        );

        self.air_body.0.push(AirBodyComponent::Constraint(
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

        assert!(
            expr.visibility().in_deductions,
            "Deduction contains an intermediate variable that is not in deductions"
        );

        self.air_body.0.push(AirBodyComponent::Deduction(
            expr.clone(),
            (!desc.is_empty()).then(|| desc.to_string()),
        ));
        self.state.add(expr, desc);
        expr.clone()
    }

    pub fn assign(&mut self, expr: &mut FeltExpr, desc: &str) -> FeltExpr {
        #[cfg(test)]
        if !self.run {
            // Cannot assert this in run mode, where we might deduce constants.
            assert!(!expr.is_const(), "Cannot assign a constant");
        }

        assert!(
            expr.in_state(),
            "The mask of the constraint must be in the trace."
        );

        let visibility = expr.visibility();
        assert!(
            visibility.in_deductions && visibility.in_constraints,
            "Assignment contains an intermediate variable that is not in both constraints and deductions"
        );

        let before = expr.clone();
        self.state.add(expr, desc);

        let constraint = expr.clone() - before.clone();
        self.air_body.0.push(AirBodyComponent::Assignment {
            constraint: constraint.clone(),
            deduction: before,
            desc: (!desc.is_empty()).then(|| desc.to_string()),
        });
        expr.clone()
    }

    pub fn let_for_deduction<V>(&mut self, var: V, desc: &str) -> V
    where
        V: AirVar,
    {
        let name = self.get_intermediate_name((!desc.is_empty()).then(|| desc.to_string()));
        let visibility = Visibility {
            in_constraints: false,
            in_deductions: true,
        };
        self.air_body.0.push(AirBodyComponent::Intermediate(
            name.clone(),
            var.prover_type(),
            var.clone().into(),
            visibility.clone(),
        ));
        var.let_(name, visibility)
    }

    pub fn let_for_constraint(&mut self, expr: FeltExpr, desc: &str) -> FeltExpr {
        assert!(
            expr.in_state(),
            "The mask of the intermediate variable for constraints must be in the trace."
        );

        let name = self.get_intermediate_name((!desc.is_empty()).then(|| desc.to_string()));
        let visibility = Visibility {
            in_constraints: true,
            in_deductions: false,
        };
        self.air_body.0.push(AirBodyComponent::Intermediate(
            name.clone(),
            Felt::r#type(),
            expr.clone().into(),
            visibility.clone(),
        ));
        expr.let_(name, visibility)
    }

    fn let_for_deduction_and_constraint<O>(&mut self, expr: O, desc: &str) -> O
    where
        O: AirVar,
    {
        assert!(
            expr.in_state(),
            "The mask of the intermediate variable for constraints must be in the trace."
        );

        let name = self.get_intermediate_name((!desc.is_empty()).then(|| desc.to_string()));
        let visibility = Visibility {
            in_constraints: true,
            in_deductions: true,
        };
        self.air_body.0.push(AirBodyComponent::Intermediate(
            name.clone(),
            expr.prover_type(),
            expr.clone().into(),
            visibility.clone(),
        ));
        expr.let_(name, visibility)
    }

    pub fn let_(&mut self, expr: FeltExpr, desc: &str) -> FeltExpr {
        self.let_for_deduction_and_constraint(expr, desc)
    }

    pub fn let_vec<O>(&mut self, vec: Vec<FeltExpr>, desc: &str) -> O
    where
        O: AirVar + From<Vec<FeltExpr>>,
    {
        let output = O::from(vec);
        self.let_for_deduction_and_constraint(output, desc)
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
            "AirFn must be inline"
        );

        if let Some(input_in_trace) = air_fn.input_in_trace() {
            if input_in_trace {
                assert!(input.in_state(), "Input should be in the trace.");
            }
        }

        // Make sure the callee is in the registry
        self.registry.add_entry(air_fn);

        let mut air_builder = Self {
            state: self.state.clone(),
            air_body: AirBody(vec![]),
            #[cfg(test)]
            row_number: self.row_number,
            #[cfg(test)]
            run: self.run,
            registry: self.registry.clone(),
            intermediate_id: self.intermediate_id.clone(),
        };
        let output = air_fn.call(&mut air_builder, (), input.clone());
        self.air_body.0.push(AirBodyComponent::Call(Call {
            air_fn_name: air_fn.name(),
            air_fn_description: air_fn.description(),
            input: input.into(),
            output: output.clone().into(),
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
            "AirFn must be a component"
        );

        assert!(
            input.in_state() && ext_input.in_state(),
            "The mask of the input to a lookup call must be in the trace."
        );

        // Make sure the callee is in the registry
        self.registry.add_entry(air_fn);

        let output_name = (!O::is_empty())
            .then(|| self.get_intermediate_name(Some(format!("{}_output", air_fn.name()))));
        let mut output = self.lookup_add_input_and_compute(
            air_fn,
            ext_input.clone(),
            input.clone(),
            output_name.clone(),
        );

        // Deduce the output if it is not empty.
        if !O::is_empty() {
            output = output.let_(
                output_name.expect("Output name not set"),
                Visibility {
                    in_constraints: false,
                    in_deductions: true,
                },
            );

            if let Some(descs) = output.get_felt_descriptions() {
                for (felt, desc) in output.as_felts_mut().into_iter().zip(descs) {
                    self.deduce(felt, &format!("{}_output_{}", air_fn.name(), desc));
                }
            } else {
                for felt in output.as_felts_mut() {
                    self.deduce(felt, &format!("{}_output", air_fn.name()));
                }
            }
        }

        self.air_body.0.push(AirBodyComponent::LookupTerm {
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

    pub fn chain_lookup_call<S>(
        &mut self,
        air_fn: &dyn AirFn<
            ExtIn = Seq,
            In = (ChainRoundVar, S),
            Out = (<Seq as ExtTable>::T, ChainRoundVar, S),
        >,
        state: S,
        num_iterations: usize,
    ) -> S
    where
        S: AirVar,
        (ChainRoundVar, S): AirVar,
        (<Seq as ExtTable>::T, ChainRoundVar, S): AirVar,
    {
        assert!(
            air_fn.trace_type() == TraceType::ChainRound,
            "AirFn must be a chain round"
        );

        assert!(
            state.in_state(),
            "The mask of the input to a chain lookup call must be in the trace."
        );

        assert!(
            !S::is_empty(),
            "The input to a chain lookup call must not be empty."
        );

        // Make sure the callee is in the registry
        self.registry.add_entry(air_fn);

        let mut output_name = "".to_string();
        let mut output = <(<Seq as ExtTable>::T, ChainRoundVar, S)>::new("".to_string(), false);
        let first_row = self.call_external_column(&Seq {}) * const_expr!(num_iterations as u32);
        let mut ext_input = first_row.clone();
        let mut input = (const_expr!(0), state);

        // Yield the input to the first round.
        self.air_body.0.push(AirBodyComponent::LookupTerm {
            relation_name: air_fn.relation_name().expect("Relation name not set"),
            felts: ext_input
                .as_felts()
                .into_iter()
                .chain(input.as_felts())
                .collect(),
            use_or_yield: UseOrYield::Yield,
        });

        // TODO(AnatG): Add all inputs to the lookup component together in one LookupAddInput.
        for i in 0..num_iterations {
            output_name =
                self.get_intermediate_name(Some(format!("{}_output_round_{}", air_fn.name(), i)));
            ext_input = first_row.clone() + const_expr!(i as u32);
            output = self.lookup_add_input_and_compute(
                air_fn,
                ext_input.clone(),
                input.clone(),
                Some(output_name.clone()),
            );

            // Prepare the input for the next round.
            input = (const_expr!(i as u32 + 1), output.2.clone());
        }

        // Deduce the output of the last round.
        output = output.let_(
            output_name,
            Visibility {
                in_constraints: false,
                in_deductions: true,
            },
        );

        if let Some(descs) = output.get_felt_descriptions() {
            for (felt, desc) in output.as_felts_mut().into_iter().zip(descs) {
                self.deduce(felt, &format!("{}_output_{}", air_fn.name(), desc));
            }
        } else {
            for felt in output.as_felts_mut() {
                self.deduce(felt, &format!("{}_output", air_fn.name()));
            }
        }

        // Use the output of the last round.
        self.air_body.0.push(AirBodyComponent::LookupTerm {
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
        let mut output = O::new(output_name.clone().unwrap_or_default(), false);
        let ext_input_option = (!E::T::is_empty()).then(|| ext_input.clone().into());
        let input_option = (!I::is_empty()).then(|| input.clone().into());

        self.air_body.0.push(AirBodyComponent::LookupAddInput {
            air_fn_name: air_fn.name(),
            ext_input: ext_input_option.clone(),
            input: input_option.clone(),
        });

        #[cfg(test)]
        if self.run {
            let mut air_builder = Self {
                state: State::default(),
                air_body: AirBody(vec![]),
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
            self.air_body
                .0
                .push(AirBodyComponent::LookupCall(LookupCall {
                    air_fn_name: air_fn.name(),
                    ext_input: ext_input_option,
                    input: input_option,
                    output_name,
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

        self.air_body
            .0
            .push(AirBodyComponent::LookupCall(LookupCall {
                air_fn_name: memory.name(),
                ext_input: Some(key.clone().into()),
                input: None,
                output_name: Some(value_name.clone()),
            }));

        #[allow(unused_mut)]
        let mut value = V::new(value_name.clone(), false);

        #[cfg(test)]
        if self.run {
            let mut air_builder = Self {
                state: State::default(),
                air_body: AirBody(vec![]),
                // This is None for the same reason as in lookup_call.
                row_number: None,
                run: self.run,
                registry: self.registry.clone(),
                // The intermediate_id is not used in run mode.
                intermediate_id: self.intermediate_id.clone(),
            };
            value = memory.lookup_call(&mut air_builder, key.clone(), ());
        }

        value.let_(
            value_name,
            Visibility {
                in_constraints: false,
                in_deductions: true,
            },
        )
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

        assert!(key.in_state(), "The key must be in the trace.");
        assert!(value.in_state(), "The value must be in the trace.");

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

        self.air_body.0.push(AirBodyComponent::LookupAddInput {
            air_fn_name: memory.name(),
            ext_input: Some(key.clone().into()),
            input: None,
        });
        self.air_body.0.push(AirBodyComponent::LookupTerm {
            relation_name: memory.relation_name().expect("Relation name not set"),
            felts: key.as_felts().into_iter().chain(value.as_felts()).collect(),
            use_or_yield: UseOrYield::Use,
        });
    }

    #[allow(unused_variables)]
    pub fn call_external_column<O>(&mut self, air_fn: &O) -> O::T
    where
        O: ExtTable + AirFn<ExtIn = (), In = (), Out = O::T>,
    {
        assert!(
            air_fn.trace_type() == TraceType::Const,
            "External columns must be constant"
        );

        // Make sure the callee is in the registry
        self.registry.add_entry(air_fn);

        #[cfg(test)]
        if self.run {
            let mut air_builder = Self {
                state: State::default(),
                air_body: AirBody(vec![]),
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
