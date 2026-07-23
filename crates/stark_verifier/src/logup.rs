use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::inv;
use stwo::core::Fraction;

pub type LogupTerm = Fraction<Var, Var>;

/// Computes the logup term for a single element.
pub fn logup_term(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    numerator: Var,
    element: &[Var],
) -> LogupTerm {
    let denominator = combine_term(context, element, interaction_elements);
    LogupTerm::new(numerator, denominator)
}

/// Computes the denominator of a logup term.
pub fn combine_term(
    context: &mut Context<impl IValue>,
    element: &[Var],
    interaction_elements: [Var; 2],
) -> Var {
    let mut element_iter = element.iter().rev();
    let mut value = *element_iter.next().unwrap();
    for elm in element_iter {
        value = eval!(context, (value) * (interaction_elements[1]));
        value = eval!(context, (value) + (*elm));
    }
    eval!(context, (value) - (interaction_elements[0]))
}

/// Computes the logup use term for the given element.
///
/// The use term is 1 / combine_term(context, element, interaction_elements).
pub fn logup_use_term(
    context: &mut Context<impl IValue>,
    element: &[Var],
    interaction_elements: [Var; 2],
) -> Var {
    let combined = combine_term(context, element, interaction_elements);
    // combined is evaluated at the random interaction elements, so it is non-zero with high
    // probability.
    inv(context, combined)
}

/// Computes the constraint polynomial for a single logup term.
pub fn single_logup_constraint(
    context: &mut Context<impl IValue>,
    term: LogupTerm,
    shifted_diff: Var,
) -> Var {
    eval!(context, ((shifted_diff) * (term.denominator)) - (term.numerator))
}

/// Computes the constraint polynomial for a pair logup term.
pub fn pair_logup_constraint(
    context: &mut Context<impl IValue>,
    term0: LogupTerm,
    term1: LogupTerm,
    shifted_diff: Var,
) -> Var {
    let denominator = eval!(context, (term0.denominator) * (term1.denominator));
    let numerator = eval!(
        context,
        ((term1.numerator) * (term0.denominator)) + ((term0.numerator) * (term1.denominator))
    );
    let term = LogupTerm::new(numerator, denominator);
    single_logup_constraint(context, term, shifted_diff)
}
