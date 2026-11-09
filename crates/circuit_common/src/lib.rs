pub mod finalize;
pub mod preprocessed;

// TODO(Anat): Take from somewhere stable.
pub const N_LANES: usize = 16;

pub struct Qm31OpsTraceGenerator {
    pub first_permutation_row: usize,
}
