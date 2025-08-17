use std::collections::HashMap;

use compiled_casm_air::compiled_structs::{
    CompiledAirVar, CompiledTraceGenIntermediate, ExternalState, LookupTerm, PaddingType,
    TraceGenStep,
};
use convert_case::{Case, Casing};
use genco::lang::{rust, Rust};
use genco::quote;
use itertools::Itertools;

use super::{deduction_consts, packed_name};
use crate::code_gen::parse::is_const_size_component;
use crate::code_gen::trace_gen::{vec_of_type, Mode, RustProverGen};
use crate::code_gen::utils::{block_doc, get_variable_name, replace_generics_with_turbofish};

impl RustProverGen {
    // TODO(Gali): Consider uniting def and impl functions.
    pub fn generate_claim_generator_struct(&self) -> rust::Tokens {
        let mut claim_generator_fields = match self.mode {
            Mode::NoInputs => quote! { pub log_size: u32, },
            Mode::PackedInputs => {
                quote! { pub packed_inputs: $(vec_of_type("PackedInputType")), }
            }
            Mode::Inputs => quote! { pub inputs: $(vec_of_type("InputType")), },
            Mode::Mults => quote! { pub mults: AtomicMultiplicityColumn, },
        };
        // TODO(Gali): Get the types of the public params from air_infra.
        for public_param in &self.public_params {
            claim_generator_fields.extend(quote! { pub $(public_param.name()): u32, });
        }
        let derive_default = (self.can_use_derive_default())
            .then(|| {
                quote! { #[derive(Default)] }
            })
            .unwrap_or_default();
        quote! {
            $derive_default
            pub struct ClaimGenerator {
                $(claim_generator_fields)
            }
        }
    }

    pub fn generate_claim_generator_impl(&self) -> rust::Tokens {
        let (add_inputs_code, self_param) = match self.mode {
            Mode::NoInputs => (quote! {}, quote! {self, }),
            Mode::Inputs => (quote! {}, quote! {mut self, }),
            Mode::PackedInputs => (
                quote! {
                    pub fn add_packed_inputs(&mut self, inputs: &[PackedInputType]) {
                        self.packed_inputs.extend(inputs);
                    }
                },
                quote! {mut self, },
            ),
            Mode::Mults => (
                quote! {
                pub fn add_input(&self, _input: &InputType) {
                    todo!() // Implement manually
                }

                pub fn add_packed_inputs(&self, packed_inputs: &[PackedInputType]) {
                    packed_inputs.into_par_iter().for_each(|packed_input| {
                        packed_input.unpack().into_iter().for_each(|input| {
                            self.add_input(&input);
                        });
                    });
                }},
                quote! {self, },
            ),
        };

        let default_mult_code = if !self.can_use_derive_default() {
            quote! {
                impl Default for ClaimGenerator {
                fn default() -> Self {
                    Self {
                        mults: AtomicMultiplicityColumn::new(1 << LOG_SIZE),
                    }
                }
            }}
        } else {
            quote! {}
        };

        let new_code = match self.mode {
            Mode::PackedInputs => quote! {
                pub fn new() -> Self {
                    Self {
                        packed_inputs: vec![],
                    }
                }
            },
            Mode::Inputs => quote! {
                pub fn new(inputs: Vec<InputType>) -> Self {
                    Self { inputs }
                }
            },
            Mode::NoInputs => {
                let builtin_segment_start = self.public_params[0].name();
                quote! {
                    pub fn new(log_size: u32, $(builtin_segment_start.clone()): u32) -> Self {
                        assert!(log_size >= LOG_N_LANES);
                        Self {
                            log_size,
                            $(builtin_segment_start),
                        }
                    }
                }
            }
            _ => quote! {},
        };

        quote! {
            $(default_mult_code)
            impl ClaimGenerator {
                $(new_code)

                pub fn write_trace(
                    $(self_param)
                    tree_builder: &mut impl TreeBuilder<SimdBackend>,
                    $(write_trace_params(&self.write_trace_context))
                ) -> (Claim, InteractionClaimGenerator)
                {
                    $(self.write_trace_body_simd())
                }

                $(add_inputs_code)
            }
        }
    }

    fn write_trace_body_simd(&self) -> rust::Tokens {
        let mut claim_fields = if is_const_size_component(&self.lists) {
            quote! {}
        } else {
            quote! {log_size,}
        };
        for public_param in &self.public_params {
            claim_fields.extend(quote! {
                $(public_param.name()): self.$(public_param.name()),
            });
        }

        let init_code = match self.mode {
            Mode::NoInputs => quote! {
               let log_size = self.log_size;
            },
            Mode::PackedInputs => quote! {
                assert!(!self.packed_inputs.is_empty());
                let n_vec_rows = self.packed_inputs.len();
                let n_rows = n_vec_rows * N_LANES;
                let packed_size = n_vec_rows.next_power_of_two();
                let log_size = packed_size.ilog2() + LOG_N_LANES;
                self.packed_inputs.resize(packed_size, *self.packed_inputs.first().unwrap());
            },
            Mode::Inputs => quote! {
                let n_rows = self.inputs.len();
                assert_ne!(n_rows, 0);
                let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
                let log_size = size.ilog2();
                self.inputs.resize(size, *self.inputs.first().unwrap());
                let packed_inputs = pack_values(&self.inputs);
            },
            Mode::Mults => quote! {
                let mults = self.mults.into_simd_vec();
            },
        };
        let mut interaction_claim_fields = match self.lists.padding_type {
            PaddingType::Enabler => quote! { n_rows, },
            _ => quote! {},
        };
        if !is_const_size_component(&self.lists) {
            interaction_claim_fields.extend(quote! { log_size, });
        }

        let sub_component_inputs = if self.contains_sub_components() {
            quote! { sub_component_inputs }
        } else {
            quote! {}
        };
        let add_inputs = self
            .add_input_mults
            .iter()
            .map(|(component_name, ..)| {
                let component_name = component_name.to_lowercase();
                quote! { sub_component_inputs.$(&component_name).iter().for_each(|inputs| {
                    $component_name$STATE_SUFFIX.add_packed_inputs(inputs);
                });}
            })
            .collect_vec();
        quote! {
            $(init_code)

            let (trace, lookup_data, $sub_component_inputs) =
                    write_trace_simd($(self.generate_write_trace_simd_args()));
            $add_inputs
            tree_builder.extend_evals(trace.to_evals());

            (
            Claim {
                $(claim_fields)
            },
            InteractionClaimGenerator {
                $(interaction_claim_fields)
                lookup_data,
            },
            )
        }
    }

    // Generates the parameters for `write_trace_simd` function.
    fn generate_write_trace_simd_params(&self) -> rust::Tokens {
        let mut params = match self.mode {
            Mode::NoInputs => quote! { log_size: u32, },
            Mode::PackedInputs | Mode::Inputs => {
                quote! { inputs: $(vec_of_type("PackedInputType")), }
            }
            Mode::Mults => quote! { mults: $(vec_of_type("PackedM31")), },
        };
        if self.lists.padding_type == PaddingType::Enabler {
            params.extend(quote! { n_rows: usize, })
        }
        for public_param in &self.public_params {
            params.extend(quote! { $(public_param.name()): u32, });
        }
        params.extend(write_trace_params(&self.write_trace_context));
        params
    }

    // Generates the arguments for `write_trace_simd` function.
    fn generate_write_trace_simd_args(&self) -> rust::Tokens {
        let mut args = match self.mode {
            Mode::NoInputs => quote! { log_size, },
            Mode::PackedInputs => quote! { self.packed_inputs, },
            Mode::Inputs => quote! { packed_inputs, },
            Mode::Mults => quote! { mults, },
        };
        if self.lists.padding_type == PaddingType::Enabler {
            args.extend(quote! { n_rows, })
        }
        for public_param in &self.public_params {
            args.extend(quote! { self.$(public_param.name()), });
        }
        args.extend(write_trace_args(&self.write_trace_context));
        args
    }

    pub fn generate_simd_write_trace_code(&self) -> rust::Tokens {
        // declare constants.
        let mut constants_def_code = quote! {};
        let constants = deduction_consts(&self.lists.deductions);
        for (ty, val) in constants.into_iter() {
            let name = get_variable_name(&ty, &val);
            constants_def_code.extend(quote! {
                let $(name) = $(replace_generics_with_turbofish(&packed_name(&ty)))::broadcast(
                    $(replace_generics_with_turbofish(&ty))::from($(val))
                );
            });
        }

        let log_size = if is_const_size_component(&self.lists) {
            quote! { LOG_SIZE }
        } else {
            quote! {log_size}
        };

        let mut preprocessed_def_code = quote! {};
        for ExternalState {
            name,
            generic_param: _,
            args,
        } in &self.lists.external_states
        {
            // Seq is the only preprocessed column that is of unfixed size.
            if name == "Seq" {
                preprocessed_def_code.extend(quote! {
                    let seq = Seq::new($(&log_size));
                });
            } else {
                preprocessed_def_code.extend(quote! {
                    let $(&get_variable_name(name.to_lowercase().as_str(), args.join("_").as_str())) = $name::new($(args.join(", ")));
                });
            }
        }

        let prelude_code = match self.mode {
            Mode::NoInputs | Mode::Mults => quote! {
            let log_n_packed_rows = $(&log_size) - LOG_N_LANES;
            },
            _ => quote! {
            let log_n_packed_rows = inputs.len().ilog2();
            let log_size = log_n_packed_rows + LOG_N_LANES;
            },
        };

        let mut init_code = (
            quote! { mut trace, mut lookup_data,},
            quote! {
                ComponentTrace::<N_TRACE_COLUMNS>::uninitialized($log_size),
                LookupData::uninitialized(log_n_packed_rows),
            },
        );

        let mut lambda_producer = (
            quote! {
                trace.par_iter_mut(),
                lookup_data.par_iter_mut(),
            },
            quote! {mut row, lookup_data, },
        );

        let mut return_tuple = (
            quote! { ComponentTrace<N_TRACE_COLUMNS>, LookupData, },
            quote! {trace, lookup_data, },
        );

        if self.contains_sub_components() {
            init_code.0.extend(quote! { mut sub_component_inputs, });
            init_code.1.extend(quote! {
                SubComponentInputs::uninitialized(log_n_packed_rows),
            });
            lambda_producer.0.extend(quote! {
                sub_component_inputs.par_iter_mut(),
            });
            lambda_producer.1.extend(quote! {
                sub_component_inputs,
            });
            return_tuple.0.extend(quote! {SubComponentInputs, });
            return_tuple.1.extend(quote! {sub_component_inputs, });
        };

        match self.mode {
            Mode::NoInputs | Mode::Mults => {}
            _ => {
                lambda_producer.0.extend(quote! {
                   inputs.into_par_iter(),
                });
                lambda_producer
                    .1
                    .extend(quote! { $(&self.lists.name)_input, });
            }
        }

        let padding = match self.lists.padding_type {
            PaddingType::Enabler => quote!(let enabler_col = Enabler::new(n_rows);),
            _ => quote!(),
        };

        let mut code = rust::Tokens::new();
        code.extend(quote! {
            // TODO(Ohad): attempt to remove this.
            #[allow(clippy::useless_conversion)]
            #[allow(unused_variables)]
            #[allow(clippy::double_parens)]
            #[allow(non_snake_case)]
            fn write_trace_simd(
                $(self.generate_write_trace_simd_params())
            ) -> ($(return_tuple.0)) {
                $(prelude_code)
                let ($(init_code.0)) = unsafe {
                    ($(init_code.1))
                };

                $(constants_def_code)
                $(preprocessed_def_code)
                $(padding)

                ($(lambda_producer.0))
                .into_par_iter()
                .enumerate()
                .for_each(
                    |(row_index,($(lambda_producer.1)))| {
                        $(self.write_trace_lambda())
                    });

                ($(return_tuple.1))
            }
            $['\n']
        });
        code
    }

    // Generates the body of the write_trace function.
    fn write_trace_lambda(&self) -> rust::Tokens {
        let const_names = &self
            .constants
            .iter()
            .map(|(ty, value)| ((ty.clone(), value.clone()), get_variable_name(ty, value)))
            .collect_vec();
        let mut write_trace_body = rust::Tokens::new();
        let mut offset = 0;
        let mut add_inputs_offsets = HashMap::new();
        for deduction in &self.lists.deductions {
            if let TraceGenStep::LookupAddInput { fn_name, .. } = deduction {
                add_inputs_offsets.insert(fn_name, 0);
            }
        }
        for ExternalState {
            name,
            generic_param: _,
            args,
        } in &self.lists.external_states
        {
            if name == "Seq" {
                write_trace_body.append(quote! {
                    let $(&name.to_lowercase()) = $(&name.to_lowercase()).packed_at(row_index);
                });
            } else {
                write_trace_body.append(quote! {
                let $(&get_variable_name(name.to_lowercase().as_str(), args.join("_").as_str())) = $(&get_variable_name(name.to_lowercase().as_str(), args.join("_").as_str())).packed_at(row_index);
                });
            }
        }

        let mut relation_data_offsets = HashMap::new();
        for relation in &self.relation_calls {
            relation_data_offsets.insert(relation, 0);
        }
        for deduction in &self.lists.deductions {
            match deduction {
                TraceGenStep::Deduction(expr) => {
                    let name = self.lists.state_names[offset].clone();
                    write_trace_body.append(quote! {
                        let $(name.clone()) = $(simd_parse_air_var(expr,const_names));
                        *row[$(offset)] = $(name);
                    });
                    offset += 1;
                }
                TraceGenStep::Intermediate(CompiledTraceGenIntermediate {
                    name,
                    r#type: _,
                    var,
                }) => {
                    write_trace_body.extend(quote! {
                        let $(name) = $(simd_parse_air_var(var,const_names));
                    });
                }
                TraceGenStep::StartBlock(msg) => {
                    write_trace_body.extend(block_doc(msg));
                }
                TraceGenStep::EndBlock => {
                    write_trace_body.extend(quote!(
                        $['\n']
                    ));
                }
                TraceGenStep::LookupTerm(LookupTerm {
                    relation_name,
                    felts,
                    ..
                }) => {
                    let offset = relation_data_offsets.get_mut(relation_name).unwrap();
                    let felts = felts
                        .iter()
                        .map(|felt| simd_parse_air_var(felt, const_names))
                        .join(", ");
                    let felts = &felts;
                    let collect_felts = quote! {
                        // TODO(Ohad): change this to not vec.
                        *lookup_data.$(relation_name.to_case(Case::Snake))_$(*offset) = [$(felts)];
                    };
                    write_trace_body.extend(collect_felts);
                    *offset += 1;
                }
                TraceGenStep::LookupAddInput { fn_name, input } => {
                    let offset = add_inputs_offsets.get_mut(fn_name).unwrap();
                    if input != &CompiledAirVar::Tuple(vec![]) {
                        write_trace_body.extend(quote! {
                            *sub_component_inputs.$(fn_name)[$(offset.to_string())] =
                                $(simd_parse_air_var(input, const_names));

                        });
                    }
                    *offset += 1;
                }
            }
        }

        // Padding code.
        write_trace_body.extend(match self.lists.padding_type {
            PaddingType::Enabler => quote! {
                *row[$(offset)] = enabler_col.packed_at(row_index);
            },
            PaddingType::Multiplicity => quote! {

                let mult_at_row = *mults.get(row_index).unwrap_or(&PackedM31::zero());
                *row[$(offset)] = mult_at_row;
                *lookup_data.mults = mult_at_row;
            },
            _ => quote!(),
        });

        write_trace_body
    }

    fn can_use_derive_default(&self) -> bool {
        self.lists.padding_type != PaddingType::Multiplicity
    }
}

const STATE_SUFFIX: &str = "_state";
fn write_trace_params(context: &[String]) -> rust::Tokens {
    let mut params = rust::Tokens::new();
    for fn_name in context {
        params.extend(quote! {
            $(fn_name)$STATE_SUFFIX: &$(fn_name)::ClaimGenerator,$("\n")
        });
    }
    params
}

fn write_trace_args(context: &[String]) -> rust::Tokens {
    let mut args = rust::Tokens::new();
    for fn_name in context {
        args.extend(quote! {
            $(fn_name)$STATE_SUFFIX,
        });
    }
    args
}

/// Parses a `CompiledAirVar` into a string for the write_trace function.
fn simd_parse_air_var(
    expr: &CompiledAirVar,
    constant_names: &[((String, String), String)],
) -> String {
    match expr {
        CompiledAirVar::Const(ty, val) => match ty.as_str() {
            // "usize" is used as index.
            // TODO(Ohad): ask anatg about this.
            "usize" => val.to_string(),
            _ => constant_names
                .iter()
                .find(|((t, v), _)| t == ty && v == val)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| {
                    let name = get_variable_name(ty, val);
                    panic!("const_{}", name)
                }),
        },
        CompiledAirVar::Var(_, id) => id.clone(),
        CompiledAirVar::State(name) => name.clone(),
        CompiledAirVar::StaticCall(id, args) => {
            // TODO(Ohad): get that information from the air infra.
            if id.starts_with("Memory") {
                let mut id = id.to_case(Case::Snake);
                id = id.replace("::", &format!("{STATE_SUFFIX}."));
                let input = simd_parse_air_var(&args[0], constant_names);
                return format!("{}({})", id, input);
            }

            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&simd_parse_air_var(arg, constant_names));
            }
            let id = id
                .replace("from_felt252", "from_packed_felt252")
                .replace("from_biguint", "from_packed_biguint");
            if id.ends_with("from_packed_felt252_array") {
                return format!("Packed{}(&{})", id, arg_str);
            }
            format!("Packed{}({})", id, arg_str)
        }
        CompiledAirVar::MethodCall(id, func, args) => {
            let func = if func == "as_felt" { "as_m31" } else { func };
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&simd_parse_air_var(arg, constant_names));
            }
            format!(
                "{}.{}({})",
                simd_parse_air_var(id, constant_names),
                func,
                arg_str
            )
        }
        CompiledAirVar::UnaryOp(op, expr) => {
            if op == "inverse" {
                return format!("({}).inverse()", simd_parse_air_var(expr, constant_names));
            }
            format!("{}({})", op, simd_parse_air_var(expr, constant_names))
        }
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            format!(
                "(({}) {} ({}))",
                simd_parse_air_var(lhs, constant_names),
                op,
                simd_parse_air_var(rhs, constant_names)
            )
        }
        CompiledAirVar::Tuple(exprs) => {
            let mut expr_str = String::new();
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    expr_str.push_str(", ");
                }
                expr_str.push_str(&simd_parse_air_var(expr, constant_names));
            }
            format!("({})", expr_str)
        }
        CompiledAirVar::Array(exprs) => {
            format!(
                "[{}]",
                exprs
                    .iter()
                    .map(|e| simd_parse_air_var(e, constant_names))
                    .join(", ")
            )
        }
        CompiledAirVar::Struct { r#type, fields } => {
            let members_code = fields
                .iter()
                .map(|(name, expr)| {
                    format!("{}: {}", name, simd_parse_air_var(expr, constant_names))
                })
                .collect::<Vec<_>>()
                .join(", ");
            let quote: genco::Tokens<Rust> = quote! {
                $(packed_name(r#type)) {
                    $(members_code),
                }
            };
            quote.to_string().unwrap()
        }
        CompiledAirVar::ExternalState(ExternalState {
            name,
            generic_param: _,
            args,
        }) => {
            if name == "Seq" {
                name.to_lowercase()
            } else {
                let args = &args.join("_");
                get_variable_name(name.to_lowercase().as_str(), args.as_str())
            }
        }
        CompiledAirVar::PublicParam(public_param) => {
            format!("PackedM31::broadcast(M31::from({public_param}))")
        }
    }
}
