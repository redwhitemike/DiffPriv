use crate::data_manipulation::anonymizable::Anonymizable;
use crate::publishing::publisher::Publisher;
use uuid::Uuid;

#[derive(Default)]
pub struct DummyPublisher {}

impl Publisher for DummyPublisher {
    fn publish<M: Anonymizable>(&mut self, _value: M, uuid: Uuid, dr: f64) {}
}
