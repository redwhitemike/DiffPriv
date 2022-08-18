use crate::data_manipulation::anonymizable::{Anonymizable, QuasiIdentifierTypes};

/// This trait lets you implement a custom noising function to add ε-differential privacy to
/// a struct that implements `Anonymizable`
/// DiffPriv already supports Laplace noise as a possible noiser
pub trait Noiser: Default + Clone + Sync {
    fn add_noise<M: Anonymizable>(&mut self, value: &M) -> Vec<QuasiIdentifierTypes>;
}
