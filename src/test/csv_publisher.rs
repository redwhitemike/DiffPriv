use crate::data_manipulation::anonymizable::Anonymizable;
use crate::publishing::publisher::Publisher;
use crate::test::csv_exporter::CsvExporter;
use uuid::Uuid;

pub struct CsvPublisher<'a> {
    exporter: &'a mut CsvExporter,
}

impl<'a> CsvPublisher<'a> {
    pub fn new(exporter: &'a mut CsvExporter) -> Self {
        Self { exporter }
    }
}

impl<'a> Publisher for CsvPublisher<'a> {
    fn publish<M: Anonymizable>(&mut self, value: M, uuid: Uuid, dr: f64) {
        self.exporter.add(value.extract_string_values(uuid, dr));
    }
}
