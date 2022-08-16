use crate::anonymization::microagg_anonymizer::MicroaggAnonymizer;
use crate::config::Config;
use crate::data_manipulation::mueller::MuellerStream;
use crate::noise::laplace::laplace_noiser::LaplaceNoiser;
use crate::publishing::kafka_publisher::KafkaPublisher;
use avro_rs::from_value;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use strm_privacy_driver::StrmPrivacyValue;

pub struct KafkaService {
    consumer: Consumer,
}

impl KafkaService {
    pub fn consume(&mut self) {
        let noiser = LaplaceNoiser::new(0.1, 3, 0.1);
        let publisher = KafkaPublisher::default();
        let mut microagg: MicroaggAnonymizer<LaplaceNoiser, MuellerStream, KafkaPublisher> =
            MicroaggAnonymizer::new(3, 20, 2, 7, 0.1, 300, 5, publisher, noiser);
        loop {
            for ms in self.consumer.poll().unwrap().iter() {
                for m in ms.messages() {
                    let mut b = &m.value[5..];
                    let confluent_bytes = &m.value[..5];
                    microagg.publisher.confluent_bytes = confluent_bytes.to_vec();
                    let mueller_value = avro_rs::from_avro_datum(
                        &MuellerStream::get_schema(MuellerStream::STRM_SCHEMA),
                        &mut b,
                        None,
                    )
                    .expect("couldn't convert message to mueller");
                    let mueller = from_value::<MuellerStream>(&mueller_value)
                        .expect("couldn't convert from value");
                    microagg.anonymize(mueller)
                }
                self.consumer
                    .consume_messageset(ms)
                    .expect("couldn't consume message");
            }
            self.consumer.commit_consumed().unwrap();
        }
    }
}

impl Default for KafkaService {
    fn default() -> Self {
        let config = Config::new(&"application.conf".to_string());

        let consumer = Consumer::from_hosts(vec![config.kafka_bootstrap.to_owned()])
            .with_topic_partitions(config.topic_in, &[0, 1])
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group("my-group".to_owned())
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .create()
            .expect("Consumer couldn't connect to bootstrap");

        Self { consumer }
    }
}
