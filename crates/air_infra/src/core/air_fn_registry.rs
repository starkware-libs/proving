use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::rc::Rc;

use serde::Serialize;
use serde_json::to_writer_pretty;

use super::air_fn::*;
use super::autogen_structs::*;
use super::state::*;
use super::variables::*;

pub const CONSTRAINT_INTERMEDIATE_VAR_PREFIX: &str = "constraint_tmp_";
pub const DEDUCTION_INTERMEDIATE_VAR_PREFIX: &str = "deduction_tmp_";

// AirFnEntry describes everything we know about an Air function.
#[derive(Debug, Clone, Serialize)]
pub struct AirFnEntry {
    pub name: String,
    pub inst_def: BTreeMap<String, String>,
    pub input: GenericAirVar,
    pub output: GenericAirVar,
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
            input: input.into(),
            output: output.into(),
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
    pub air_fns: Rc<RefCell<BTreeMap<String, AirFnEntry>>>,
    #[serde(skip)]
    pub intermediate_vars_index: Rc<RefCell<usize>>,
}

impl AirFnRegistry {
    pub fn new<I, O>(air_fn: &dyn AirFn<In = I, Out = O>) -> Self
    where
        I: AirVar,
        O: AirVar,
    {
        // Create the registry.
        let registry = Self {
            air_fns: Rc::new(RefCell::new(BTreeMap::new())),
            intermediate_vars_index: Rc::new(RefCell::new(0)),
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
        assert!(self.air_fns.borrow().get(&air_fn.name()).is_some());

        let mut air_builder = AirBuilder {
            state: State::default(),
            air_body: vec![],
            run: true,
            registry: self.clone(),
        };
        let output = match air_fn.trace_type() {
            TraceType::Inline => {
                assert!(input.in_state(), "Input must be in the trace");
                air_fn.call(&mut air_builder, input)
            }
            TraceType::Component => air_fn.lookup_call(&mut air_builder, input),
            TraceType::Const => air_fn.call(&mut air_builder, input),
        };
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
        (air_builder, input, output)
    }

    pub(super) fn get_intermediate_var_index(&self) -> String {
        let mut index = self.intermediate_vars_index.borrow_mut();
        let index_as_str = format!("{}", *index);
        *index += 1;
        index_as_str
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
            .expect("Air function not found")
            .clone()
    }

    pub fn get_codegen_air_fn<I, O>(&self, air_fn: &dyn AirFn<In = I, Out = O>) -> AutogenLists
    where
        I: AirVar,
        O: AirVar,
    {
        let entry = self.get_air_fn_entry(air_fn);
        Self::compile_codegen_air_fn(entry.air_body, entry.input)
    }

    // Transforms the air body and input of an air function into the autogen format.
    fn compile_codegen_air_fn(
        air_body: Vec<AirBodyComponent>,
        input: GenericAirVar,
    ) -> AutogenLists {
        let mut constraints = vec![];
        let mut deductions = vec![];

        for component in air_body {
            match component {
                AirBodyComponent::Constraint(constraint) => {
                    constraints.push(ConstraintOrIntermediate::InInstanceConstraint(
                        constraint.into(),
                    ));
                }
                AirBodyComponent::Assignment {
                    constraint,
                    deduction,
                } => {
                    constraints.push(ConstraintOrIntermediate::InInstanceConstraint(
                        constraint.into(),
                    ));
                    deductions.push(TraceGenerationStep::Deduction(deduction.into()));
                }
                AirBodyComponent::Deduction(deduction) => {
                    deductions.push(TraceGenerationStep::Deduction(deduction.into()));
                }
                AirBodyComponent::DeductionIntermediate(name, var) => {
                    deductions.push(TraceGenerationStep::Intermediate(name, var.into()));
                }
                AirBodyComponent::ConstraintIntermediate(name, var) => {
                    constraints.push(ConstraintOrIntermediate::Intermediate(name, var.into()));
                }
                AirBodyComponent::Call(f) => {
                    let lists = Self::compile_codegen_air_fn(f.air_body, f.input_arg);
                    constraints.extend(lists.constraints);
                    deductions.extend(lists.deductions);
                }
                AirBodyComponent::LookupCall(call) => {
                    deductions.push(TraceGenerationStep::Lookup {
                        fn_name: call.air_fn_name,
                        input: call.input_arg.into(),
                        output_name: call.output_name,
                    });
                }
                AirBodyComponent::LookupConstraint(constraint) => {
                    constraints.push(ConstraintOrIntermediate::LookupConstraint {
                        fn_name: constraint.air_fn_name.clone(),
                        input_felts: constraint
                            .input_felts
                            .iter()
                            .map(|x| (*x).clone().into())
                            .collect(),
                        output_felts: constraint
                            .output_felts
                            .iter()
                            .map(|x| (*x).clone().into())
                            .collect(),
                    });
                }
            }
        }

        AutogenLists {
            input: input.into(),
            constraints,
            deductions,
        }
    }

    pub(super) fn get_deduction_intermediate_var_name(&self) -> String {
        let index = self.get_intermediate_var_index();
        format!("{}{}", DEDUCTION_INTERMEDIATE_VAR_PREFIX, index)
    }

    pub(super) fn get_constraint_intermediate_var_name(&self) -> String {
        let index = self.get_intermediate_var_index();
        format!("{}{}", CONSTRAINT_INTERMEDIATE_VAR_PREFIX, index)
    }
}
