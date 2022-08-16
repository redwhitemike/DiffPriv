use crate::data_manipulation::anonymizable::{Anonymizable, QuasiIdentifierTypes};

pub trait Noiser: Default + Clone + Sync {
    fn add_noise<M: Anonymizable>(&mut self, value: &M) -> Vec<QuasiIdentifierTypes>;
}
