use crate::config::Config;
use crate::data_manipulation::anonymizable::Anonymizable;
use crate::data_manipulation::mueller::MuellerStream;
use crate::publishing::publisher::Publisher;
use avro_rs::{to_avro_datum, to_value};
use kafka::producer::{Producer, Record, RequiredAcks};
use std::time::Duration;
use strm_privacy_driver::StrmPrivacyValue;
use uuid::Uuid;

pub struct KafkaPublisher {
    producer: Producer,
    pub confluent_bytes: Vec<u8>,
    topic_out: String,
    published: i32,
}

impl KafkaPublisher {
    pub fn new(confluent_bytes: Vec<u8>) -> Self {
        Self {
            confluent_bytes,
            ..Default::default()
        }
    }
}

impl Default for KafkaPublisher {
    fn default() -> Self {
        let config = Config::new(&"application.conf".to_string());
        let producer = Producer::from_hosts(vec![config.kafka_bootstrap.to_owned()])
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create()
            .expect("Producer couldn't connect to kafka bootstrap");

        let confluent_bytes: Vec<u8> = Vec::new();

        Self {
            confluent_bytes,
            producer,
            topic_out: config.topic_out,
            published: 0,
        }
    }
}

impl Publisher for KafkaPublisher {
    fn publish<M: Anonymizable>(&mut self, value: M, uuid: Uuid, dr: f64) {
        let converted_value = to_value(value).unwrap();
        let mut datum = to_avro_datum(
            &MuellerStream::get_schema(MuellerStream::STRM_SCHEMA),
            converted_value,
        )
        .unwrap();

        let mut send = self.confluent_bytes.to_vec();
        send.append(&mut datum);

        self.published += 1;

        debug!("{}", self.published);

        match self
            .producer
            .send(&Record::from_value(self.topic_out.as_str(), send))
        {
            Ok(_) => {
                println!("Produced!")
            }
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }
}
