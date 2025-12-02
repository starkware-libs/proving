use std::collections::HashMap;

use compiled_casm_air::compiled_structs::{
    CompiledAirVar, CompiledTraceGenIntermediate, LookupTerm, PaddingType, TraceGenStep,
};
use convert_case::{Case, Casing};
use genco::lang::{rust, Rust};
use genco::quote;
use indexmap::IndexMap;
use itertools::Itertools;

use super::{deduction_consts, packed_name};
use crate::code_gen::trace_gen::{vec_of_type, Mode, RustProverGen};
use crate::code_gen::utils::{
    block_doc, get_variable_name, is_const_size_component, make_preprocessed_column_id,
    relations_used_or_yielded, replace_generics_with_turbofish,
};

impl RustProverGen {
    // TODO(Gali): Consider uniting def and impl functions.
    pub fn generate_claim_generator_struct(&self) -> rust::Tokens {
        let mut claim_generator_fields = match self.mode {
            Mode::NoInputs => quote! { pub log_size: u32, },
            Mode::PackedInputs => {
                quote! { pub packed_inputs: $(vec_of_type("PackedInputType")), }
            }
            Mode::Inputs => quote! { pub inputs: $(vec_of_type("InputType")), },
            Mode::Mults => {
                if is_const_size_component(&self.air_fn) {
                    let n_input_columns = self.air_fn.input_const_columns.len();
                    let n_relations = self.air_fn.relation_names.len();
                    quote! {
                        pub mults: [AtomicMultiplicityColumn; $(n_relations)],
                        input_to_row: HashMap<[M31; $(n_input_columns)], usize>,
                        preprocessed_trace: Arc<PreProcessedTrace>
                    }
                } else {
                    quote! { pub mults: DashMap<InputType, AtomicU32>, }
                }
            }
        };
        // TODO(Gali): Get the types of the public params from air_infra.
        for public_param in &self.air_fn.public_params {
            claim_generator_fields.extend(quote! { pub $(public_param.name()): u32, });
        }
        let derive_default = (!is_const_size_component(&self.air_fn))
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
                    pub fn add_packed_inputs(&mut self, inputs: &[PackedInputType], _relation_name: &str) {
                        self.packed_inputs.extend(inputs);
                    }
                },
                quote! {mut self, },
            ),
            Mode::Mults => {
                let mut add_inputs_code = if is_const_size_component(&self.air_fn) {
                    quote! {
                        pub fn add_input(&self, input: &InputType, relation_name: &str) {
                            let rel_ind = $(format!("{:?}", self.air_fn.relation_names)).iter().position(|r| *r == relation_name).unwrap();
                            self.mults[rel_ind]
                                .increase_at((*self.input_to_row.get(input).unwrap()).try_into().unwrap());
                        }
                    }
                } else {
                    quote! {
                        pub fn add_input(&self, input: &InputType, _relation_name: &str) {
                            self.mults
                                .entry(*input)
                                .or_insert_with(|| AtomicU32::new(0))
                                .fetch_add(1, Ordering::Relaxed);
                        }
                    }
                };
                add_inputs_code.extend(quote! {
                    $['\n']
                    pub fn add_packed_inputs(
                        &self,
                        packed_inputs: &[PackedInputType],
                        relation_name: &str,
                    ) {
                        packed_inputs.into_par_iter().for_each(|packed_input| {
                            packed_input.unpack().into_par_iter().for_each(|input| {
                                self.add_input(&input, relation_name);
                            });
                        });
                    }
                });
                (add_inputs_code, quote! {self, })
            }
        };

        let new_code = match self.mode {
            Mode::Mults if is_const_size_component(&self.air_fn) => {
                let column_ids_code = rust::Tokens::from_iter(
                    Itertools::intersperse(
                        self.air_fn
                            .input_const_columns
                            .iter()
                            .map(make_preprocessed_column_id),
                        quote! { , },
                    )
                    .flatten(),
                );
                quote! {
                    pub fn new(preprocessed_trace: Arc<PreProcessedTrace>) -> Self {
                        let mults = from_fn(|_| AtomicMultiplicityColumn::new(1 << LOG_SIZE));
                        let column_ids = [$(column_ids_code)];

                        Self {
                            mults,
                            input_to_row: make_input_to_row(&preprocessed_trace, column_ids),
                            preprocessed_trace
                        }
                    }
                }
            }
            Mode::PackedInputs | Mode::Mults => quote! {
                pub fn new() -> Self {
                    Self::default()
                }
            },
            Mode::Inputs => quote! {
                pub fn new(inputs: Vec<InputType>) -> Self {
                    Self { inputs }
                }
            },
            Mode::NoInputs => {
                let builtin_segment_start = self.air_fn.public_params[0].name();
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
        };

        let is_empty_code = match self.mode {
            Mode::PackedInputs => quote! {
                pub fn is_empty(&self) -> bool {
                    self.packed_inputs.is_empty()
                }
            },
            Mode::Mults if !is_const_size_component(&self.air_fn) => quote! {
                pub fn is_empty(&self) -> bool {
                    self.mults.is_empty()
                }
            },
            _ => quote! {},
        };

        quote! {
            $("\n")

            impl ClaimGenerator {
                $(new_code)

                $(is_empty_code)

                pub fn write_trace(
                    $(self_param)
                    tree_builder: &mut impl TreeBuilder<SimdBackend>,
                    $(write_trace_params(&self.air_fn.sub_components))
                ) -> (Claim, InteractionClaimGenerator)
                {
                    $(self.write_trace_body_simd())
                }

                $(add_inputs_code)
            }
        }
    }

    fn write_trace_body_simd(&self) -> rust::Tokens {
        let mut claim_fields = if is_const_size_component(&self.air_fn) {
            quote! {}
        } else {
            quote! {log_size,}
        };
        for public_param in &self.air_fn.public_params {
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
            Mode::Mults => {
                if is_const_size_component(&self.air_fn) {
                    quote! { let mults = self.mults.into_iter().map(|v| v.into_simd_vec()).collect::<Vec<_>>(); }
                } else {
                    quote! {
                        let mut inputs_mults = self
                            .mults
                            .iter()
                            .map(|entry| (*entry.key(), M31(entry.value().load(Ordering::Relaxed))))
                            .collect::<Vec<_>>();

                        inputs_mults.sort_by_key(|(input, _)| input.0);

                        let (mut inputs, mut mults) = inputs_mults.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();

                        // Calculate component size.
                        let n_rows = inputs.len();
                        assert_ne!(n_rows, 0);
                        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
                        let log_size = size.ilog2();

                        // Pad component.
                        inputs.resize(size, *inputs.first().unwrap());
                        mults.resize(size, M31::zero());

                        let packed_inputs = pack_values(&inputs);
                        let packed_mults = pack_values(&mults);
                    }
                }
            }
        };
        let mut interaction_claim_fields = match self.air_fn.padding_type {
            PaddingType::Enabler => quote! { n_rows, },
            _ => quote! {},
        };
        if !is_const_size_component(&self.air_fn) {
            interaction_claim_fields.extend(quote! { log_size, });
        }

        let sub_component_inputs = if self.contains_sub_components() {
            quote! { sub_component_inputs }
        } else {
            quote! {}
        };
        let add_inputs = self
            .air_fn
            .n_inputs_added_per_relation
            .iter()
            .map(|(relation_name, (component_name, _))| {
                quote! { sub_component_inputs.$(relation_name.to_case(Case::Snake)).iter().for_each(|inputs| {
                    $component_name$STATE_SUFFIX.add_packed_inputs(inputs, $(format!("\"{}\"", relation_name)));
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
            Mode::Mults => {
                if !is_const_size_component(&self.air_fn) {
                    quote! {
                        inputs: $(vec_of_type("PackedInputType")),
                        mults: $(vec_of_type("PackedM31")),
                    }
                } else {
                    quote! {
                        preprocessed_trace: &PreProcessedTrace,
                        mults: Vec<$(vec_of_type("PackedM31"))>,
                    }
                }
            }
        };
        if self.air_fn.padding_type == PaddingType::Enabler {
            params.extend(quote! { n_rows: usize, })
        }
        for public_param in &self.air_fn.public_params {
            params.extend(quote! { $(public_param.name()): u32, });
        }
        params.extend(write_trace_params(&self.air_fn.sub_components));
        params
    }

    // Generates the arguments for `write_trace_simd` function.
    fn generate_write_trace_simd_args(&self) -> rust::Tokens {
        let mut args = match self.mode {
            Mode::NoInputs => quote! { log_size, },
            Mode::PackedInputs => quote! { self.packed_inputs, },
            Mode::Inputs => quote! { packed_inputs, },
            Mode::Mults => {
                if is_const_size_component(&self.air_fn) {
                    quote! { &self.preprocessed_trace, mults, }
                } else {
                    quote! { packed_inputs, packed_mults, }
                }
            }
        };
        if self.air_fn.padding_type == PaddingType::Enabler {
            args.extend(quote! { n_rows, })
        }
        for public_param in &self.air_fn.public_params {
            args.extend(quote! { self.$(public_param.name()), });
        }
        args.extend(write_trace_args(&self.air_fn.sub_components));
        args
    }

    pub fn generate_simd_write_trace_code(&self) -> rust::Tokens {
        // declare constants.
        let mut constants_def_code = quote! {};
        let constants = deduction_consts(&self.air_fn.deductions);
        for (ty, val) in constants.into_iter() {
            let name = get_variable_name(&ty, &val);
            constants_def_code.extend(quote! {
                let $(name) = $(replace_generics_with_turbofish(&packed_name(&ty)))::broadcast(
                    $(replace_generics_with_turbofish(&ty))::from($(val))
                );
            });
        }

        let log_size = if is_const_size_component(&self.air_fn) {
            quote! { LOG_SIZE }
        } else {
            quote! {log_size}
        };

        let mut preprocessed_def_code = quote! {};
        for external_col_id in &self.air_fn.external_states {
            // Seq is the only preprocessed column that is of unfixed size.
            if external_col_id == "Seq" {
                preprocessed_def_code.extend(quote! {
                    let seq = Seq::new($(&log_size));
                });
            } else {
                preprocessed_def_code.extend(quote! {
                    let $(external_col_id) = preprocessed_trace.get_column(&$(make_preprocessed_column_id(external_col_id)));
                });
            }
        }

        let prelude_code = match self.mode {
            Mode::NoInputs => quote! {
            let log_n_packed_rows = $(&log_size) - LOG_N_LANES;
            },
            Mode::Mults if is_const_size_component(&self.air_fn) => quote! {
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
            quote! {row, lookup_data, },
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
            Mode::NoInputs => {}
            Mode::Mults if is_const_size_component(&self.air_fn) => {}
            _ => {
                lambda_producer.0.extend(quote! {
                   inputs.into_par_iter(),
                });
                lambda_producer
                    .1
                    .extend(quote! { $(&self.air_fn.name)_input, });
            }
        }

        let padding = match self.air_fn.padding_type {
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
        for deduction in &self.air_fn.deductions {
            if let TraceGenStep::LookupAddInput { relation_name, .. } = deduction {
                add_inputs_offsets.insert(relation_name, 0);
            }
        }
        for external_col_id in &self.air_fn.external_states {
            write_trace_body.append(quote! {
                let $(external_col_id.to_lowercase()) = $(external_col_id.to_lowercase()).packed_at(row_index);
            });
        }

        let mut relation_data_offsets = HashMap::new();
        for relation in relations_used_or_yielded(&self.air_fn) {
            relation_data_offsets.insert(relation, 0);
        }
        for deduction in &self.air_fn.deductions {
            match deduction {
                TraceGenStep::Deduction(expr) => {
                    let name = self.air_fn.state_names[offset].clone();
                    write_trace_body.append(quote! {
                        let $(name.clone()) = $(simd_parse_air_var(expr, const_names));
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
                TraceGenStep::LookupAddInput {
                    relation_name,
                    input,
                    ..
                } => {
                    let offset = add_inputs_offsets.get_mut(relation_name).unwrap();
                    if input != &CompiledAirVar::Tuple(vec![]) {
                        write_trace_body.extend(quote! {
                            *sub_component_inputs.$(relation_name.to_case(Case::Snake))[$(offset.to_string())] =
                                $(simd_parse_air_var(input, const_names));

                        });
                    }
                    *offset += 1;
                }
            }
        }

        // Padding code.
        write_trace_body.extend(match self.air_fn.padding_type {
            PaddingType::Enabler => quote! {
                *row[$(offset)] = enabler_col.packed_at(row_index);
            },
            PaddingType::Multiplicity => {
                if !is_const_size_component(&self.air_fn) {
                    quote! {
                        let mult_at_row = *mults.get(row_index).unwrap_or(&PackedM31::zero());
                        *row[$(offset)] = mult_at_row;
                        *lookup_data.mults_0 = mult_at_row;
                    }
                } else {
                    let mut code = quote! {};
                    for i in 0..self.air_fn.relation_names.len() {
                        code.extend(quote! {
                            let mult = &mults[$i];
                            let mult_at_row = *mult.get(row_index).unwrap_or(&PackedM31::zero());
                            *row[$(offset + i)] = mult_at_row;
                            *lookup_data.mults_$i = mult_at_row;
                        });
                    }
                    code
                }
            }
            _ => quote!(),
        });

        write_trace_body
    }
}

const STATE_SUFFIX: &str = "_state";
fn write_trace_params(sub_components: &IndexMap<String, PaddingType>) -> rust::Tokens {
    let mut params = rust::Tokens::new();
    for (name, padding_type) in sub_components {
        if padding_type == &PaddingType::Multiplicity {
            params.extend(quote! {
                $name$STATE_SUFFIX: &$name::ClaimGenerator,$("\n")
            });
        } else {
            params.extend(quote! {
                $name$STATE_SUFFIX: &mut $name::ClaimGenerator,$("\n")
            });
        }
    }
    params
}

fn write_trace_args(sub_components: &IndexMap<String, PaddingType>) -> rust::Tokens {
    let mut args = rust::Tokens::new();
    for (name, _) in sub_components {
        args.extend(quote! {
            $name$STATE_SUFFIX,
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
            // TODO(AnatG): get that information from the air infra.
            let functions_requiring_state_object = [
                "BlakeRound::deduce_output",
                "MemoryAddressToId::deduce_output",
                "MemoryIdToBig::deduce_output",
            ];
            if functions_requiring_state_object.contains(&id.as_str()) {
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
        CompiledAirVar::ExternalState(col_id) => col_id.to_lowercase(),
        CompiledAirVar::PublicParam(public_param) => {
            format!("PackedM31::broadcast(M31::from({public_param}))")
        }
    }
}
