use crate::data_manipulation::anonymizable::Anonymizable;
use uuid::Uuid;

/// Generic trait for publishing the anonymized data
/// Anonymizable also contains Serialize to make it easy to
/// convert a value for specific publishers
pub trait Publisher {
    fn publish<M: Anonymizable>(&mut self, value: M, uuid: Uuid, dr: f64);
}
