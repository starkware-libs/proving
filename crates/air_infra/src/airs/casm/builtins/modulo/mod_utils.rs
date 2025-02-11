use std::array::from_fn;

use inst_def::InstDef;
use prover_types::cpu::FELT252_BITS_PER_WORD;

use crate::airs::casm::casm_state::*;
use crate::airs::casm::common::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::read_positive::*;
use crate::core::felt252_id_memory::verify_equal::*;

// Length of each modulo builtin word in bits.
pub const MOD_BUILTIN_WORD_BIT_LEN: usize = 96;
// Number of words composing a mod_builtin number.
pub const MOD_BUILTIN_N_WORDS: usize = 4;
// Number of subwords in a word when each FELT252_BITS_PER_WORD bits is a subword.
pub const N_SUBWORDS_IN_WORD: usize = MOD_BUILTIN_WORD_BIT_LEN.div_ceil(FELT252_BITS_PER_WORD);
// Length of the last subword in each word in bits.
pub const LAST_SUBWORD_BIT_LEN: usize =
    MOD_BUILTIN_WORD_BIT_LEN - (N_SUBWORDS_IN_WORD - 1) * FELT252_BITS_PER_WORD;
// Total number of subwords in a mod_builtin number.
pub const TOTAL_SUBWORDS: usize = MOD_BUILTIN_N_WORDS * N_SUBWORDS_IN_WORD;
// Number of inputs for each instance, i.e. p0,...,p3, values_ptr, offsets_ptr, n.
pub const N_VAR_INPUTS: usize = 7;

#[derive(Clone, Debug, InstDef)]
pub struct ModUtils {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

// A function that recieves a starting address and an instance number from a mod builtin function
// and returns the values of p,a,b,c, while also verifying that the values n, offsets_ptr,
// values_ptr and p are consistent with that of the previous instance.
impl AirFn for ModUtils {
    type ExtIn = ();
    type In = (CasmAddress, FeltExpr);
    type Out = [[Felt252Expr; MOD_BUILTIN_N_WORDS]; 4];

    fn call(&self, ab: &mut AirBuilder, _: (), (first_addr, instance_num): Self::In) -> Self::Out {
        // Read a 96 bit word from the memory which is already range checked to be 96 bits.
        let read_word = ReadPositive {
            memory: self.memory.clone(),
            num_bits: N_SUBWORDS_IN_WORD * FELT252_BITS_PER_WORD,
        };

        // Instance 0 is a special case because it doesn't have a previous instance to compare to.
        let mut is_instance_0 =
            ab.let_for_deduction(instance_num.clone().eq(const_expr!(0)), "is_instance_0");
        let is_instance_0 = ab.deduce(is_instance_0.as_felt_mut(), "is_instance_0");
        ab.constrain(
            is_instance_0.clone() * (is_instance_0.clone() - const_expr!(1)),
            "is_instance_0 is 0 or 1.",
        );
        ab.constrain(
            is_instance_0.clone() * instance_num.clone(),
            "is_instance_0 is 0 when instance_num is not 0.",
        );
        // Calculate the starting address of the previous instance and the current one.
        let input_var_addr_start_prev = first_addr.var.clone()
            + const_expr!(N_VAR_INPUTS as u32)
                * (instance_num.clone() - const_expr!(1) + is_instance_0.clone());
        let input_var_addr_start =
            first_addr.var + const_expr!(N_VAR_INPUTS as u32) * instance_num.clone();

        let (p_addr_prev, p_addr): (Vec<CasmAddress>, Vec<CasmAddress>) = (0..MOD_BUILTIN_N_WORDS)
            .map(|i| {
                (
                    CasmAddress::new(
                        input_var_addr_start_prev.clone() + const_expr!(i as u32),
                        &format!("p_prev{}", i),
                    ),
                    CasmAddress::new(
                        input_var_addr_start.clone() + const_expr!(i as u32),
                        &format!("p{}", i),
                    ),
                )
            })
            .unzip();

        let [(values_ptr_addr_prev, values_ptr_addr), (offsets_ptr_addr_prev, offsets_ptr_addr), (n_addr_prev, n_addr)] =
            from_fn(|j| {
                (
                    input_var_addr_start_prev.clone()
                        + const_expr!((j + MOD_BUILTIN_N_WORDS) as u32),
                    input_var_addr_start.clone() + const_expr!((j + MOD_BUILTIN_N_WORDS) as u32),
                )
            });

        let (p_val, p_ids): (Vec<Felt252Expr>, Vec<FeltExpr>) = p_addr
            .into_iter()
            .map(|addr| ab.call(&read_word, addr))
            .unzip();

        // Read inputs from memory.
        let (values_ptr_val_felt252, values_ptr_id) = ab.call(
            &ReadPositive {
                memory: self.memory.clone(),
                num_bits: ADDRESS_BITS,
            },
            CasmAddress::new(values_ptr_addr, "values_ptr"),
        );
        let values_ptr_val = felt252_to_m31(values_ptr_val_felt252, ADDRESS_BITS);
        let [offsets_ptr_val, offsets_ptr_val_prev, n_val, n_val_prev_nominal] = [
            (offsets_ptr_addr, "offsets_ptr"),
            (offsets_ptr_addr_prev, "offsets_ptr_prev"),
            // n is not an address, but it should be no greater than the maximal address.
            (n_addr, "n"),
            (n_addr_prev, "n_prev"),
        ]
        .into_iter()
        .map(|(addr, name)| {
            self.memory
                .read_address(ab, CasmAddress::new(addr, name))
                .var
        })
        .collect::<Vec<_>>()
        .try_into()
        .expect("Conversion to array failed.");

        // If instance 0, then n_val_prev = 1, else n_val_prev = n_val_prev_nominal
        let n_val_prev = ab.let_for_constraint(
            n_val_prev_nominal * (const_expr!(1) - is_instance_0.clone()) + is_instance_0.clone(),
            "n_val_prev",
        );
        // Condition for block reset, i.e. when the input variables can progress arbitrarily.
        let block_reset_condition = n_val_prev.clone() - const_expr!(1);
        // Constrain the values of n, offsets_ptr, values_ptr to be consistent with the previous
        // instance.
        ab.constrain(
            block_reset_condition.clone() * (n_val_prev.clone() - const_expr!(1) - n_val.clone()),
            "Progression of n between instances.",
        );

        ab.constrain(
            block_reset_condition.clone()
                * (offsets_ptr_val.clone() - const_expr!(3) - offsets_ptr_val_prev.clone()),
            "Progression of offsets_ptr between instances.",
        );

        ab.call(
            &MemCondVerifyEqualKnownId {
                memory: self.memory.clone(),
            },
            (
                CasmAddress::new(values_ptr_addr_prev, "values_ptr_prev"),
                values_ptr_id,
                block_reset_condition.clone(),
            ),
        );
        for i in 0..MOD_BUILTIN_N_WORDS {
            ab.call(
                &MemCondVerifyEqualKnownId {
                    memory: self.memory.clone(),
                },
                (
                    p_addr_prev[i].clone(),
                    p_ids[i].clone(),
                    block_reset_condition.clone(),
                ),
            );
        }

        // Read the offsets and values of a,b,c.
        let offsets_val: [FeltExpr; 3] = from_fn(|j| {
            self.memory.read_rel_imm(
                ab,
                CasmAddress::new(
                    offsets_ptr_val.clone() + const_expr!(j as u32),
                    &format!("offsets_{}", ['a', 'b', 'c'][j]),
                ),
            )
        });

        let vars_val: [[Felt252Expr; MOD_BUILTIN_N_WORDS]; 3] = from_fn(|j| {
            from_fn(|k| {
                ab.call(
                    &read_word,
                    CasmAddress::new(
                        values_ptr_val.clone() + offsets_val[j].clone() + const_expr!(k as u32),
                        &format!("{}{}", ['a', 'b', 'c'][j], k),
                    ),
                )
                .0
            })
        });

        [
            p_val
                .try_into()
                .expect("p_val should have MOD_BUILTIN_N_WORDS elements."),
            vars_val[0].clone(),
            vars_val[1].clone(),
            vars_val[2].clone(),
        ]
    }
}
