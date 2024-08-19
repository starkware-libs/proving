use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::rc::Rc;

use indexmap::IndexMap;
use serde::Serialize;
use serde_json::to_writer_pretty;

use super::air_fn::*;
use super::compiled_structs::*;
use super::expressions::felt_expr::*;
use super::state::*;
use super::variables::*;

pub const INTERMEDIATE_VAR_PREFIX: &str = "tmp_";

// AirFnEntry describes everything we know about an Air function.
#[derive(Debug, Clone, Serialize)]
pub struct AirFnEntry {
    pub name: String,
    pub inst_def: IndexMap<String, String>,
    pub input: AirVarImpl,
    pub input_num_of_felts: usize,
    pub output: AirVarImpl,
    pub output_felts: Vec<FeltExpr>,
    pub trace_type: TraceType,
    pub air_body: Vec<AirBodyComponent>,
}

impl AirFnEntry {
    pub(super) fn new<I, O>(registry: &AirFnRegistry, air_fn: &dyn AirFn<In = I, Out = O>) -> Self
    where
        I: AirVar,
        O: AirVar,
    {
        let (air_builder, input, output) = registry.build_air(air_fn);
        let entry = Self {
            name: air_fn.name(),
            inst_def: air_fn.inst_def(),
            input: input.clone().into(),
            input_num_of_felts: input.as_felts().len(),
            output: output.clone().into(),
            output_felts: output.as_felts(),
            trace_type: air_fn.trace_type(),
            air_body: air_builder.air_body.clone(),
        };
        air_builder
            .registry
            .air_fns
            .borrow_mut()
            .insert(air_fn.name(), entry.clone());
        entry
    }
}

// AirFnRegistry is created for a specific air function. It keeps all the air function entries
// for the air function and its subroutines.
#[derive(Debug, Clone, Serialize)]
pub struct AirFnRegistry {
    pub air_fns: Rc<RefCell<HashMap<String, AirFnEntry>>>,
    #[serde(skip)]
    pub intermediate_index: Rc<RefCell<usize>>,
}

impl AirFnRegistry {
    pub fn new<I, O>(air_fn: &dyn AirFn<In = I, Out = O>) -> Self
    where
        I: AirVar,
        O: AirVar,
    {
        // Create the registry.
        let registry = Self {
            air_fns: Rc::new(RefCell::new(HashMap::new())),
            intermediate_index: Rc::new(RefCell::new(0)),
        };
        // Add the function to the registry.
        AirFnEntry::new(&registry, air_fn);
        registry
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
            TraceType::Const => air_fn.call(&mut air_builder, input),
        };

        // Make sure that the output is in the state.
        assert!(output.in_state());
        (air_builder.state, output)
    }

    // Builds the air function on a default input in order to create an air function entry for it.
    fn build_air<I, O>(&self, air_fn: &dyn AirFn<In = I, Out = O>) -> (AirBuilder, I, O)
    where
        I: AirVar,
        O: AirVar,
    {
        let input = I::new(format!("{}_input", air_fn.name()));
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
            TraceType::Component => air_fn.lookup_call(&mut air_builder, input.clone()),

            // For constant AirFns the value of <output> is meaningless, as we don't
            // output any constraints or deductions. It just has to be of the correct type.
            TraceType::Const => O::new(format!("{}_output", air_fn.name())),
        };

        // Make sure that the output is a variable or a felt expression.
        let _output_felts = output.as_felts();
        (air_builder, input, output)
    }

    // Dumps the registry to a file.
    pub fn dump_to_file(&self, file_name: &str) {
        let mut path = Self::project_root();
        path.push(format!("src/{}", file_name));
        let file = File::create(path).expect("Unable to create file");
        let mut writer = BufWriter::new(file);
        to_writer_pretty(&mut writer, self).expect("serialization failed");
        writer.flush().expect("flush failed");
        writer.write_all(b"\n").expect("write failed");
    }

    fn project_root() -> PathBuf {
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
    }

    pub fn get_air_fn_entry<I, O>(&self, air_fn: &dyn AirFn<In = I, Out = O>) -> AirFnEntry
    where
        I: AirVar,
        O: AirVar,
    {
        self.air_fns
            .borrow()
            .get(&air_fn.name())
            .unwrap_or_else(|| panic!("Air function {} not found", air_fn.name()))
            .clone()
    }

    pub fn get_compiled_air_fn<I, O>(&self, air_fn: &dyn AirFn<In = I, Out = O>) -> CompiledAirFn
    where
        I: AirVar,
        O: AirVar,
    {
        let entry = self.get_air_fn_entry(air_fn);
        let (deductions, constraints) = Self::compile_air_fn(entry.air_body);
        CompiledAirFn {
            input: entry.input.into(),
            output: entry.output.into(),
            input_num_of_felts: entry.input_num_of_felts,
            output_felts: entry
                .output_felts
                .into_iter()
                .map(CompiledAirVar::from)
                .collect(),
            constraints,
            deductions,
        }
    }

    // Transforms the air body of an air function into the compiled air fn format.
    fn compile_air_fn(
        air_body: Vec<AirBodyComponent>,
    ) -> (Vec<TraceGenStep>, Vec<ConstraintEvalStep>) {
        let mut constraints = vec![];
        let mut deductions = vec![];

        for component in air_body {
            match component {
                AirBodyComponent::Constraint(constraint) => {
                    constraints.push(ConstraintEvalStep::InInstanceConstraint(constraint.into()));
                }
                AirBodyComponent::Assignment {
                    constraint,
                    deduction,
                } => {
                    constraints.push(ConstraintEvalStep::InInstanceConstraint(constraint.into()));
                    deductions.push(TraceGenStep::Deduction(deduction.into()));
                }
                AirBodyComponent::Deduction(deduction) => {
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
                    let (new_deductions, new_constraints) = Self::compile_air_fn(f.air_body);
                    constraints.extend(new_constraints);
                    deductions.extend(new_deductions);
                }
                AirBodyComponent::LookupCall(call) => {
                    deductions.push(TraceGenStep::Lookup {
                        fn_name: call.air_fn_name,
                        input: call.input_arg.into(),
                        output_name: call.output_name,
                    });
                }
                AirBodyComponent::LookupConstraint(constraint) => {
                    constraints.push(ConstraintEvalStep::LookupConstraint {
                        fn_name: constraint.air_fn_name,
                        input_felts: constraint
                            .input_felts
                            .into_iter()
                            .map(|x| x.into())
                            .collect(),
                        output_felts: constraint
                            .output_felts
                            .into_iter()
                            .map(|x| x.into())
                            .collect(),
                    });
                }
                AirBodyComponent::AccessExternalColumn(access) => {
                    deductions.push(TraceGenStep::AccessExternalColumn {
                        fn_name: access.air_fn_name.clone(),
                        output_name: access.output_name.clone(),
                    });
                    constraints.push(ConstraintEvalStep::AccessExternalColumn {
                        fn_name: access.air_fn_name,
                        output_name: access.output_name,
                    })
                }
            }
        }

        (deductions, constraints)
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
