use std::fs::File;
use std::time::{Instant, SystemTime};

use csv::Reader;
use polars::prelude::*;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use uuid::Uuid;

use crate::analysis::analyser::Analyser;
use crate::anonymization::microagg_anonymizer::MicroaggAnonymizer;
use crate::data_manipulation::anonymizable::QuasiIdentifierType::Integer;
use crate::data_manipulation::anonymizable::QuasiIdentifierTypes::{Interval, Nominal};
use crate::data_manipulation::anonymizable::{
    Anonymizable, QuasiIdentifierTypes, SensitiveAttribute,
};
use crate::noise::laplace::laplace_noiser::LaplaceNoiser;
use crate::publishing::csv_publisher::CsvPublisher;
use crate::test::csv_exporter::CsvExporter;
use crate::test::environment::Environment;
use crate::test::metrics::Metrics;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct EnrichedRow {
    pub id: String,
    pub gender: String,
    pub age: i32,
    pub run_id: i32,
    pub running: bool,
    pub speed: f32,
    pub duration: f32,
    pub distance: f32,
    pub side: String,
    pub bout: i32,
    pub freq: f32,
    pub walk_ratio: f32,
    pub start: String,
    pub end: String,
    #[serde(skip_deserializing, default = "default_time")]
    time_generated: SystemTime,
}

fn default_time() -> SystemTime {
    SystemTime::now()
}

impl Default for EnrichedRow {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            gender: "".to_string(),
            age: 0,
            run_id: 0,
            running: false,
            speed: 0.0,
            duration: 0.0,
            distance: 0.0,
            side: "".to_string(),
            bout: 0,
            freq: 0.0,
            walk_ratio: 0.0,
            start: "".to_string(),
            end: "".to_string(),
            time_generated: SystemTime::now(),
        }
    }
}

impl Anonymizable for EnrichedRow {
    fn quasi_identifiers(&self) -> Vec<QuasiIdentifierTypes> {
        let age = Interval((Integer(self.age), Integer(33), Integer(85), 1));
        let gender_string = self.gender.to_owned();
        let gender = match gender_string.as_str() {
            "male" => (0, 1, 1),
            "female" => (1, 1, 1),
            _ => panic!("Not all categories covered"),
        };

        let gender = Nominal(gender);

        vec![age, gender]
    }

    fn update_quasi_identifiers(&self, mut qi: Vec<QuasiIdentifierTypes>) -> Self {
        let mut update = self.clone();
        let gender_qi = qi.pop().unwrap().extract_value();
        let age_qi = qi.pop().unwrap().extract_value();

        match gender_qi {
            Integer(0) => update.gender = String::from("male"),
            Integer(1) => update.gender = String::from("female"),
            _ => panic!("Not all categories covered"),
        }

        if let Integer(age) = age_qi {
            update.age = age
        }

        update
    }

    fn sensitive_value(&self) -> SensitiveAttribute {
        SensitiveAttribute::String(self.id.to_owned())
    }

    fn extract_string_values(&self, uuid: Uuid, dr: f64) -> Vec<String> {
        vec![
            uuid.to_string(),
            dr.to_string(),
            self.id.to_string(),
            self.gender.to_string(),
            self.age.to_string(),
            self.run_id.to_string(),
            self.running.to_string(),
            self.speed.to_string(),
            self.duration.to_string(),
            self.distance.to_string(),
            self.side.to_string(),
            self.bout.to_string(),
            self.freq.to_string(),
            self.walk_ratio.to_string(),
            self.start.to_string(),
            self.end.to_string(),
        ]
    }

    fn get_timestamp(&self) -> SystemTime {
        self.time_generated
    }
}

pub struct CsvImporter {
    file_reader: Reader<File>,
}

impl CsvImporter {
    pub fn new(file_reader: Reader<File>) -> Self {
        Self { file_reader }
    }

    pub fn convert<A: Anonymizable + DeserializeOwned>(&mut self, env: Environment) -> Result<()> {
        let (dataset_name, exporter, mut microagg) = Self::setup(&env);
        let duration = Instant::now();

        let data_info = LazyCsvReader::new("datasets/datatypes_adult_1_class_50K.csv".into())
            .has_header(true)
            .finish()
            .unwrap()
            .collect()
            .unwrap();

        for line in self.file_reader.deserialize() {
            let row: A = line?;
            let df = JsonReader::new(row).finish().unwrap();
            let new_df = df.vstack(&data_info).unwrap();
            println!("{}", new_df);
            assert!()
        }

        println!("cluster remaining: {}", microagg.cluster_set.len());

        if env.publish_remaining_tuples {
            microagg
                .cluster_set
                .into_iter()
                .for_each(|(_, mut cluster)| {
                    cluster.publish_all(&mut microagg.publisher, &mut microagg.analysers)
                });
        }

        let elapsed = duration.elapsed();
        let mut metrics = Metrics::default();

        println!("duration: {:?}", elapsed);
        metrics.execution_time = elapsed.as_millis();

        microagg
            .analysers
            .iter()
            .for_each(|analyser| match analyser {
                Analyser::Mse(mse) => {
                    metrics.mse = mse.calculate_mse();
                    println!("MSE: {}", mse.calculate_mse());
                }
                Analyser::Sse(sse) => {
                    metrics.sse = sse.total_info_loss();
                    println!("SSE: {}", sse.total_info_loss())
                }
                Analyser::PublishingDelay(publishing_delay) => {
                    metrics.publishing_delay =
                        publishing_delay.calculate_average_delay().as_nanos();
                    println!(
                        "Average publishing delay: {:?}",
                        publishing_delay.calculate_average_delay()
                    )
                }
                Analyser::ClusterAnalyser(cluster_analyser) => {
                    metrics.clusters_created = cluster_analyser.create_counter;
                    metrics.clusters_deleted = cluster_analyser.delete_counter;
                    println!(
                        "Clusters created: {} | Clusters removed: {}",
                        cluster_analyser.create_counter, cluster_analyser.delete_counter
                    )
                }
                Analyser::DisclosureRiskAnalyser(disclosure) => {
                    metrics.disclosure_risk = disclosure.calculate_disclosure_risk();
                    println!(
                        "Disclosure risk: {}",
                        disclosure.calculate_disclosure_risk()
                    )
                }
            });

        exporter.export()?;
        println!("metrics: {:?}", metrics);
        std::fs::write(
            format!("{}.json", dataset_name),
            serde_json::to_string_pretty(&metrics).unwrap(),
        )?;
        println!("-------------------");
        Ok(())
    }

    fn setup(
        env: &Environment,
    ) -> (
        String,
        CsvExporter,
        MicroaggAnonymizer<LaplaceNoiser, A, CsvPublisher>,
    ) {
        let noiser = LaplaceNoiser::new(env.eps, env.k, env.noise_thr);
        let dataset_name = format!(
            "{}_{}_{}_{}_{}_{}_{}_{}_{}_{}",
            env.dataset.extract().export,
            env.k,
            env.k_max,
            env.l,
            env.c,
            env.diff_thres,
            env.eps,
            env.delta,
            env.buff_size,
            env.noise_thr
        );

        let mut exporter = CsvExporter::new(format!("{}.csv", dataset_name), env.dataset);
        let publisher = CsvPublisher::new(&mut exporter);
        let mut microagg: MicroaggAnonymizer<LaplaceNoiser, A, CsvPublisher> =
            MicroaggAnonymizer::new(
                env.k,
                env.k_max,
                env.l,
                env.c,
                env.diff_thres,
                env.delta,
                env.buff_size,
                publisher,
                noiser,
            );

        println!("starting anonymization with k: {}| k_max:{}| l: {}| c: {}| eps: {}| diff_thres: {}, delta: {}| noise_thr: {}| buff_size: {}",
                 env.k,
                 env.k_max,
                 env.l,
                 env.c,
                 env.eps,
                 env.diff_thres,
                 env.delta,
                 env.noise_thr,
                 env.k * 3
        );
        (dataset_name, exporter, microagg)
    }
}
